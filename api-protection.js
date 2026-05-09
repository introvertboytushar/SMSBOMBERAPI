// api/api-protection.js
// শুধু Origin check + Token check (Rate limit নেই)

import crypto from 'crypto';

// ==================== CONFIG ====================
const CONFIG = {
  ALLOWED_ORIGINS: [
    'https://sms-bomber-it.vercel.app',
    'https://customsms-it.vercel.app',
    'https://smsbomber.introvertboytushar.workers.dev',
    // 'http://localhost:5500', // local dev এ uncomment করো
  ],

  get SECRET_KEY() {
    const key = process.env.SECRET_KEY;
    if (!key) throw new Error('SECRET_KEY environment variable is not set!');
    return key;
  },

  TOKEN_EXPIRY_MS: 2 * 60 * 1000, // ২ মিনিট
};

// ==================== HELPERS ====================

function createHMAC(data, secret) {
  return crypto.createHmac('sha256', secret).update(data).digest('hex');
}

function setCORSHeaders(req, res) {
  const origin = req.headers['origin'];
  if (origin && CONFIG.ALLOWED_ORIGINS.some(o => origin.startsWith(o))) {
    res.setHeader('Access-Control-Allow-Origin', origin);
  }
  res.setHeader('Access-Control-Allow-Methods', 'POST, GET, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, X-Auth-Token, X-User-ID');
}

// ==================== ORIGIN CHECK ====================

function checkOrigin(req) {
  const origin = req.headers['origin'] || req.headers['referer'] || '';
  if (!origin) return { allowed: false, reason: 'No origin. Direct API access not allowed.' };
  const isAllowed = CONFIG.ALLOWED_ORIGINS.some(o => origin.startsWith(o));
  if (!isAllowed) return { allowed: false, reason: `Origin '${origin}' is not allowed.` };
  return { allowed: true };
}

// ==================== TOKEN ====================

function verifyToken(tokenHeader) {
  if (!tokenHeader) return { valid: false, reason: 'Token missing' };

  const parts = tokenHeader.split('.');
  if (parts.length !== 2) return { valid: false, reason: 'Invalid token format' };

  const [timestamp, signature] = parts;
  const ts = parseInt(timestamp, 10);
  if (isNaN(ts)) return { valid: false, reason: 'Invalid token' };

  if (Date.now() - ts > CONFIG.TOKEN_EXPIRY_MS) return { valid: false, reason: 'Token expired' };

  const expectedSig = createHMAC(timestamp, CONFIG.SECRET_KEY);
  if (signature !== expectedSig) return { valid: false, reason: 'Invalid token signature' };

  return { valid: true };
}

// ==================== TOKEN ENDPOINT ====================
// GET request এ call করো — client এখান থেকে token নেবে

export function getTokenForClient(req, res) {
  setCORSHeaders(req, res);
  if (req.method === 'OPTIONS') return res.status(204).end();

  const originCheck = checkOrigin(req);
  if (!originCheck.allowed) {
    return res.status(403).json({ error: 'Access denied', message: originCheck.reason });
  }

  const timestamp = Date.now().toString();
  const signature = createHMAC(timestamp, CONFIG.SECRET_KEY);
  const token = `${timestamp}.${signature}`;

  return res.json({ token, expiresIn: CONFIG.TOKEN_EXPIRY_MS });
}

// ==================== PROTECT MIDDLEWARE ====================
// POST request এর শুরুতে call করো — false হলে block হয়েছে

export function protect(req, res, next) {
  setCORSHeaders(req, res);
  if (req.method === 'OPTIONS') { res.status(204).end(); return; }

  // Origin check
  const originCheck = checkOrigin(req);
  if (!originCheck.allowed) {
    res.status(403).json({ error: 'Access denied', message: originCheck.reason });
    return;
  }

  // Token check
  const tokenCheck = verifyToken(req.headers['x-auth-token']);
  if (!tokenCheck.valid) {
    res.status(401).json({ error: 'Unauthorized', message: tokenCheck.reason });
    return;
  }

  // ✅ সব ঠিক আছে
  if (typeof next === 'function') next();
}

export { CONFIG };
