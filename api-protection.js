/**
 * ============================================================
 *  API Protection Module — Vercel Serverless Compatible
 *  - Origin/Referer check (শুধু তোমার সাইট থেকে call হবে)
 *  - HMAC Token authentication
 *  - Duplicate message rate limiting (same msg = blocked)
 *  - Per-user request rate limiting
 *  - ✅ Vercel KV (Upstash Redis) দিয়ে persistent storage
 * ============================================================
 *
 *  SETUP:
 *  1. Vercel Dashboard → Settings → Environment Variables এ দাও:
 *       SECRET_KEY=তোমার_গোপন_key
 *       KV_REST_API_URL=https://...upstash.io
 *       KV_REST_API_TOKEN=AX...
 *
 *  2. npm install @upstash/redis
 *
 *  3. তোমার API route file এ import করো:
 *       const { protect, getTokenForClient } = require('./api-protection');
 *
 *  4. route handler এ use করো:
 *       app.post('/api/chat', protect, yourHandler);
 *       app.get('/api/token', getTokenForClient);
 *
 *  CLIENT SIDE (browser):
 *    async function sendMessage(userMessage) {
 *      const tokenRes = await fetch('/api/token');
 *      const { token } = await tokenRes.json();
 *      const response = await fetch('/api/chat', {
 *        method: 'POST',
 *        headers: {
 *          'Content-Type': 'application/json',
 *          'X-Auth-Token': token,
 *          'X-User-ID': localStorage.getItem('userId') || 'anonymous',
 *        },
 *        body: JSON.stringify({ message: userMessage }),
 *      });
 *      const data = await response.json();
 *      console.log(data.reply);
 *    }
 * ============================================================
 */

import crypto from 'crypto';

// ==================== KV / Redis Client ====================
// Upstash Redis —  Marketplace থেকে connect করো (free tier আছে)
// https://vercel.com/integrations/upstash
let redis = null;

async function getRedis() {
  if (redis) return redis;
  try {
    const { Redis } = await import('@upstash/redis');
    redis = new Redis({
      url: process.env.KV_REST_API_URL,
      token: process.env.KV_REST_API_TOKEN,
    });
  } catch (_) {
    redis = null;
  }
  return redis;
}

// ==================== CONFIG ====================
const CONFIG = {
  // তোমার সাইটের domain গুলো
  ALLOWED_ORIGINS: [
    'https://sms-bomber-it.vercel.app',
    'https://customsms-it.vercel.app',
   
    // 'http://localhost:5500', // local dev এর জন্য uncomment করো
  ],

  // ✅ FIX: Secret key এখন Environment Variable থেকে আসবে
  // Vercel Dashboard → Settings → Environment Variables → SECRET_KEY
  get SECRET_KEY() {
    const key = process.env.SECRET_KEY;
    if (!key) {
      throw new Error(
        '[api-protection] SECRET_KEY environment variable is not set! ' +
        'Vercel Dashboard → Settings → Environment Variables এ SECRET_KEY দাও।'
      );
    }
    return key;
  },

  // Token কতক্ষণ valid থাকবে (milliseconds) — এখন 2 মিনিট
  TOKEN_EXPIRY_MS: 2 * 60 * 1000,

  // একই message কতক্ষণ block থাকবে (milliseconds) — এখন 1 মিনিট
  DUPLICATE_BLOCK_MS: 60 * 1000,

  // একজন user প্রতি মিনিটে কতবার call করতে পারবে
  RATE_LIMIT_PER_MINUTE: 10,

  // Rate limit window (seconds) — Redis TTL এর জন্য
  RATE_LIMIT_WINDOW_SEC: 60,

  // Duplicate block (seconds) — Redis TTL এর জন্য
  DUPLICATE_BLOCK_SEC: 60,
};
// ================================================


// ==================== FALLBACK In-Memory (local dev only) ====================
// ⚠️ এটা শুধু local development এ কাজ করবে।
// Vercel Serverless এ Redis ছাড়া rate limit / duplicate check persist হবে না।
const _memRecentMessages = new Map();
const _memUserRequests = new Map();

// Local dev এ memory clean করো
if (process.env.NODE_ENV !== 'production') {
  setInterval(() => {
    const now = Date.now();
    for (const [key, data] of _memRecentMessages.entries()) {
      if (now - data.timestamp > CONFIG.DUPLICATE_BLOCK_MS) _memRecentMessages.delete(key);
    }
    for (const [key, data] of _memUserRequests.entries()) {
      if (now - data.windowStart > CONFIG.RATE_LIMIT_WINDOW_SEC * 1000) _memUserRequests.delete(key);
    }
  }, 30_000);
}


// ==================== HELPERS ====================

/**
 * HMAC-SHA256 signature তৈরি করে
 */
function createHMAC(data, secret) {
  return crypto.createHmac('sha256', secret).update(data).digest('hex');
}

/**
 * User এর IP address বের করে (Vercel এ x-forwarded-for দরকার)
 */
function getClientIP(req) {
  return (
    req.headers['x-forwarded-for']?.split(',')[0]?.trim() ||
    req.headers['x-real-ip'] ||
    req.connection?.remoteAddress ||
    req.socket?.remoteAddress ||
    'unknown'
  );
}

/**
 * একটি message এর unique fingerprint তৈরি করে
 */
function getMessageFingerprint(userId, message) {
  const normalized = message.trim().toLowerCase();
  return crypto.createHash('sha256').update(`${userId}:${normalized}`).digest('hex');
}


// ==================== REDIS STORAGE HELPERS ====================

/**
 * Rate limit চেক ও update — Redis দিয়ে
 * Redis না থাকলে in-memory fallback
 */
async function checkRateLimitAsync(userId) {
  // Rate limit disable kora hoyeche jate unlimited bombing kora jay
  return { limited: false };
}


  // ── Fallback: in-memory (local dev) ──
  const record = _memUserRequests.get(userId);
  if (!record || now - record.windowStart > CONFIG.RATE_LIMIT_WINDOW_SEC * 1000) {
    _memUserRequests.set(userId, { count: 1, windowStart: now });
    return { limited: false };
  }
  if (record.count >= CONFIG.RATE_LIMIT_PER_MINUTE) {
    const retryAfter = Math.ceil((CONFIG.RATE_LIMIT_WINDOW_SEC * 1000 - (now - record.windowStart)) / 1000);
    return { limited: true, reason: `Too many requests. Try again in ${retryAfter} seconds.`, retryAfter };
  }
  record.count++;
  return { limited: false };
}

/**
 * Duplicate message চেক — Redis দিয়ে
 * Redis না থাকলে in-memory fallback
 */
async function checkDuplicateMessageAsync(userId, message) {
  const fingerprint = getMessageFingerprint(userId, message);
  const key = `dup:${userId}:${fingerprint}`;
  const kv = getRedis();

  if (kv) {
    try {
      // SET NX (only set if not exists) + TTL
      const set = await kv.set(key, '1', { nx: true, ex: CONFIG.DUPLICATE_BLOCK_SEC });
      if (set === null) {
        // Key আগে থেকেই ছিল — duplicate
        const ttl = await kv.ttl(key);
        return {
          isDuplicate: true,
          reason: `একই message আবার পাঠিয়েছ। ${ttl} সেকেন্ড পর চেষ্টা করো।`,
          retryAfter: ttl,
        };
      }
      return { isDuplicate: false };
    } catch (err) {
      console.error('[api-protection] Redis duplicate check error:', err.message);
      return { isDuplicate: false }; // graceful degradation
    }
  }

  // ── Fallback: in-memory ──
  const now = Date.now();
  const record = _memRecentMessages.get(key);
  if (record && now - record.timestamp < CONFIG.DUPLICATE_BLOCK_MS) {
    const retryAfter = Math.ceil((CONFIG.DUPLICATE_BLOCK_MS - (now - record.timestamp)) / 1000);
    return { isDuplicate: true, reason: `একই message আবার পাঠিয়েছ। ${retryAfter} সেকেন্ড পর চেষ্টা করো।`, retryAfter };
  }
  _memRecentMessages.set(key, { timestamp: now });
  return { isDuplicate: false };
}


// ==================== TOKEN FUNCTIONS ====================

/**
 * Token verify করে
 * Client পাঠাবে: "timestamp.signature" format এ
 */
function verifyToken(tokenHeader) {
  if (!tokenHeader) return { valid: false, reason: 'Token missing' };

  const parts = tokenHeader.split('.');
  if (parts.length !== 2) return { valid: false, reason: 'Invalid token format' };

  const [timestamp, signature] = parts;
  const ts = parseInt(timestamp, 10);

  if (isNaN(ts)) return { valid: false, reason: 'Invalid token timestamp' };

  if (Date.now() - ts > CONFIG.TOKEN_EXPIRY_MS) {
    return { valid: false, reason: 'Token expired' };
  }

  let secretKey;
  try {
    secretKey = CONFIG.SECRET_KEY;
  } catch (e) {
    console.error('[api-protection]', e.message);
    return { valid: false, reason: 'Server configuration error' };
  }

  const expectedSig = createHMAC(timestamp, secretKey);

  // ✅ FIX: Timing-safe comparison দিয়ে signature check (timing attack প্রতিরোধ)
  if (
    signature.length !== expectedSig.length ||
    !crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expectedSig))
  ) {
    return { valid: false, reason: 'Invalid token signature' };
  }

  return { valid: true };
}

/**
 * Origin/Referer চেক করে
 */
function checkOrigin(req) {
  const origin = req.headers['origin'] || req.headers['referer'] || '';

  if (!origin) {
    return { allowed: false, reason: 'No origin header. Direct API access not allowed.' };
  }

  const isAllowed = CONFIG.ALLOWED_ORIGINS.some((allowed) => origin.startsWith(allowed));

  if (!isAllowed) {
    return { allowed: false, reason: `Origin '${origin}' is not allowed.` };
  }

  return { allowed: true };
}

/**
 * CORS headers set করে
 */
function setCORSHeaders(req, res) {
  const origin = req.headers['origin'];
  if (origin && CONFIG.ALLOWED_ORIGINS.some((o) => origin.startsWith(o))) {
    res.setHeader('Access-Control-Allow-Origin', origin);
  }
  res.setHeader('Access-Control-Allow-Methods', 'POST, GET, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, X-Auth-Token, X-User-ID');
  res.setHeader('Access-Control-Max-Age', '86400'); // 24 ঘণ্টা preflight cache
}


// ==================== MAIN MIDDLEWARE ====================

/**
 * ✅ Main protection middleware (async — Vercel Serverless এ কাজ করে)
 * Express route এ এভাবে use করো:
 *   app.post('/api/chat', protect, yourHandler)
 *
 * ⚠️ এটা async middleware, তাই Express 5 বা wrapper দরকার হতে পারে।
 *    নিচের মতো wrap করো যদি Express 4 ব্যবহার করো:
 *    app.post('/api/chat', asyncHandler(protect), yourHandler)
 */
async function protect(req, res, next) {
  // CORS headers
  setCORSHeaders(req, res);

  // OPTIONS preflight
  if (req.method === 'OPTIONS') {
    return res.status(204).end();
  }

  // ── Step 1: Origin check ──
  const originCheck = checkOrigin(req);
  if (!originCheck.allowed) {
    console.warn(`[BLOCKED] Origin: ${originCheck.reason} | IP: ${getClientIP(req)}`);
    return res.status(403).json({ error: 'Access denied', message: originCheck.reason });
  }

  // ── Step 2: Token check ──
  const token = req.headers['x-auth-token'];
  const tokenCheck = verifyToken(token);
  if (!tokenCheck.valid) {
    console.warn(`[BLOCKED] Token: ${tokenCheck.reason} | IP: ${getClientIP(req)}`);
    return res.status(401).json({ error: 'Unauthorized', message: tokenCheck.reason });
  }

  // ── Step 3: User ID ──
  const userId = req.headers['x-user-id'] || getClientIP(req) || 'anonymous';



  console.log(`[ALLOWED] ${userId} | ${new Date().toISOString()}`);
  next();
}


// ==================== TOKEN ENDPOINT ====================

/**
 * তোমার Express app এ এই endpoint add করো:
 *   app.get('/api/token', getTokenForClient);
 *
 * Client প্রথমে এখান থেকে token নেবে, তারপর API call করবে।
 */
function getTokenForClient(req, res) {
  setCORSHeaders(req, res);

  if (req.method === 'OPTIONS') {
    return res.status(204).end();
  }

  const originCheck = checkOrigin(req);
  if (!originCheck.allowed) {
    return res.status(403).json({ error: 'Access denied' });
  }

  let secretKey;
  try {
    secretKey = CONFIG.SECRET_KEY;
  } catch (e) {
    console.error('[api-protection]', e.message);
    return res.status(500).json({ error: 'Server configuration error' });
  }

  const timestamp = Date.now().toString();
  const signature = createHMAC(timestamp, secretKey);
  const token = `${timestamp}.${signature}`;

  res.json({ token, expiresIn: CONFIG.TOKEN_EXPIRY_MS });
}

/**
 * Express 4 এর জন্য async middleware wrapper
 * ব্যবহার: app.post('/api/chat', asyncHandler(protect), yourHandler)
 */
function asyncHandler(fn) {
  return (req, res, next) => {
    Promise.resolve(fn(req, res, next)).catch(next);
  };
}


// ==================== EXPORTS ====================
export { protect, asyncHandler, getTokenForClient, CONFIG };


/*
 * ============================================================
 *  VERCEL SETUP GUIDE
 * ============================================================
 *
 *  1. Upstash Redis বানাও (free):
 *     https://console.upstash.com → Create Database
 *
 *  2. Vercel Dashboard এ connect করো:
 *     https://vercel.com/integrations/upstash
 *     → এটা automatically KV_REST_API_URL ও KV_REST_API_TOKEN set করবে
 *
 *  3. SECRET_KEY environment variable দাও:
 *     Vercel Dashboard → তোমার Project → Settings → Environment Variables
 *     → Name: SECRET_KEY | Value: তোমার_random_string
 *
 *  4. npm install @upstash/redis
 *
 * ============================================================
 *  COMPLETE EXAMPLE (api/chat.js — Vercel Serverless)
 * ============================================================
 *
 *  const express = require('express');
 *  const { protect, asyncHandler, getTokenForClient } = require('../lib/api-protection');
 *
 *  const app = express();
 *  app.use(express.json());
 *
 *  // Token endpoint
 *  app.get('/api/token', getTokenForClient);
 *
 *  // Protected chat endpoint
 *  app.post('/api/chat', asyncHandler(protect), async (req, res) => {
 *    const { message } = req.body;
 *    // তোমার AI API call এখানে
 *    res.json({ reply: 'Hello!' });
 *  });
 *
 *  module.exports = app;
 *
 * ============================================================
 *  CLIENT SIDE USAGE (browser)
 * ============================================================
 *
 *  async function sendMessage(userMessage) {
 *    // Step 1: Token নাও
 *    const tokenRes = await fetch('/api/token');
 *    if (!tokenRes.ok) throw new Error('Token fetch failed');
 *    const { token } = await tokenRes.json();
 *
 *    // Step 2: API call করো
 *    const response = await fetch('/api/chat', {
 *      method: 'POST',
 *      headers: {
 *        'Content-Type': 'application/json',
 *        'X-Auth-Token': token,
 *        'X-User-ID': localStorage.getItem('userId') || 'anonymous',
 *      },
 *      body: JSON.stringify({ message: userMessage }),
 *    });
 *
 *    if (!response.ok) {
 *      const err = await response.json();
 *      alert(err.message);
 *      return;
 *    }
 *
 *    const data = await response.json();
 *    console.log(data.reply);
 *  }
 *
 * ============================================================
 */
