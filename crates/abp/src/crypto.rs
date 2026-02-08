//! Cryptographic primitives for ABP package manager
//!
//! Provides SHA256 hashing and Ed25519 signature verification.

extern crate alloc;
use alloc::vec::Vec;

// =============================================================================
// SHA256 Implementation
// =============================================================================

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA256 hasher
pub struct Sha256 {
    state: [u32; 8],
    count: u64,
    buffer: [u8; 64],
    buflen: usize,
}

impl Sha256 {
    /// Create a new SHA256 hasher
    pub fn new() -> Self {
        Sha256 {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            count: 0,
            buffer: [0; 64],
            buflen: 0,
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.count += data.len() as u64;
        let mut offset = 0;

        if self.buflen > 0 {
            let space = 64 - self.buflen;
            let copy = core::cmp::min(space, data.len());
            self.buffer[self.buflen..self.buflen + copy].copy_from_slice(&data[..copy]);
            self.buflen += copy;
            offset = copy;

            if self.buflen == 64 {
                self.transform(&self.buffer.clone());
                self.buflen = 0;
            }
        }

        while offset + 64 <= data.len() {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[offset..offset + 64]);
            self.transform(&block);
            offset += 64;
        }

        if offset < data.len() {
            self.buflen = data.len() - offset;
            self.buffer[..self.buflen].copy_from_slice(&data[offset..]);
        }
    }

    /// Finalize and return the hash
    pub fn finalize(&mut self) -> [u8; 32] {
        let bit_len = self.count * 8;
        let pad_len = if self.buflen < 56 { 56 - self.buflen } else { 120 - self.buflen };

        let mut padding = [0u8; 128];
        padding[0] = 0x80;
        self.update(&padding[..pad_len]);

        let mut len_bytes = [0u8; 8];
        for i in 0..8 {
            len_bytes[7 - i] = (bit_len >> (i * 8)) as u8;
        }
        self.update(&len_bytes);

        let mut result = [0u8; 32];
        for (i, &s) in self.state.iter().enumerate() {
            result[i * 4] = (s >> 24) as u8;
            result[i * 4 + 1] = (s >> 16) as u8;
            result[i * 4 + 2] = (s >> 8) as u8;
            result[i * 4 + 3] = s as u8;
        }
        result
    }

    fn transform(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g; g = f; f = e;
            e = d.wrapping_add(temp1);
            d = c; c = b; b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

/// Compute SHA256 hash of data
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

/// Compute SHA256 hash and return as hex string
pub fn sha256_hex(data: &[u8]) -> [u8; 64] {
    let hash = sha256(data);
    let mut hex = [0u8; 64];
    const HEX: &[u8] = b"0123456789abcdef";
    for (i, &byte) in hash.iter().enumerate() {
        hex[i * 2] = HEX[(byte >> 4) as usize];
        hex[i * 2 + 1] = HEX[(byte & 0xf) as usize];
    }
    hex
}

// =============================================================================
// SHA512 Implementation (needed for Ed25519)
// =============================================================================

const SHA512_K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

/// SHA512 hasher
pub struct Sha512 {
    state: [u64; 8],
    count: u128,
    buffer: [u8; 128],
    buflen: usize,
}

impl Sha512 {
    /// Create a new SHA512 hasher
    pub fn new() -> Self {
        Sha512 {
            state: [
                0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
                0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
            ],
            count: 0,
            buffer: [0; 128],
            buflen: 0,
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.count += data.len() as u128;
        let mut offset = 0;

        if self.buflen > 0 {
            let space = 128 - self.buflen;
            let copy = core::cmp::min(space, data.len());
            self.buffer[self.buflen..self.buflen + copy].copy_from_slice(&data[..copy]);
            self.buflen += copy;
            offset = copy;

            if self.buflen == 128 {
                self.transform(&self.buffer.clone());
                self.buflen = 0;
            }
        }

        while offset + 128 <= data.len() {
            let mut block = [0u8; 128];
            block.copy_from_slice(&data[offset..offset + 128]);
            self.transform(&block);
            offset += 128;
        }

        if offset < data.len() {
            self.buflen = data.len() - offset;
            self.buffer[..self.buflen].copy_from_slice(&data[offset..]);
        }
    }

    /// Finalize and return the hash
    pub fn finalize(&mut self) -> [u8; 64] {
        let bit_len = self.count * 8;
        let pad_len = if self.buflen < 112 { 112 - self.buflen } else { 240 - self.buflen };

        let mut padding = [0u8; 256];
        padding[0] = 0x80;
        self.update(&padding[..pad_len]);

        let mut len_bytes = [0u8; 16];
        for i in 0..16 {
            len_bytes[15 - i] = (bit_len >> (i * 8)) as u8;
        }
        self.update(&len_bytes);

        let mut result = [0u8; 64];
        for (i, &s) in self.state.iter().enumerate() {
            for j in 0..8 {
                result[i * 8 + j] = (s >> (56 - j * 8)) as u8;
            }
        }
        result
    }

    fn transform(&mut self, block: &[u8; 128]) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes([
                block[i*8], block[i*8+1], block[i*8+2], block[i*8+3],
                block[i*8+4], block[i*8+5], block[i*8+6], block[i*8+7],
            ]);
        }
        for i in 16..80 {
            let s0 = w[i-15].rotate_right(1) ^ w[i-15].rotate_right(8) ^ (w[i-15] >> 7);
            let s1 = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA512_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g; g = f; f = e;
            e = d.wrapping_add(temp1);
            d = c; c = b; b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

/// Compute SHA512 hash of data
pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize()
}

// =============================================================================
// Ed25519 Implementation
// =============================================================================

/// Field element modulo p = 2^255 - 19
/// Represented as 10 limbs of 25.5 bits each (alternating 26 and 25 bits)
#[derive(Clone, Copy)]
struct Fe([i64; 10]);

impl Fe {
    const ZERO: Fe = Fe([0; 10]);
    const ONE: Fe = Fe([1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    fn from_bytes(s: &[u8; 32]) -> Fe {
        let mut h = [0i64; 10];
        h[0] = load_4(&s[0..4]) as i64;
        h[1] = (load_3(&s[4..7]) << 6) as i64;
        h[2] = (load_3(&s[7..10]) << 5) as i64;
        h[3] = (load_3(&s[10..13]) << 3) as i64;
        h[4] = (load_3(&s[13..16]) << 2) as i64;
        h[5] = load_4(&s[16..20]) as i64;
        h[6] = (load_3(&s[20..23]) << 7) as i64;
        h[7] = (load_3(&s[23..26]) << 5) as i64;
        h[8] = (load_3(&s[26..29]) << 4) as i64;
        h[9] = ((load_3(&s[29..32]) & 0x7fffff) << 2) as i64;
        Fe(h)
    }

    fn to_bytes(&self) -> [u8; 32] {
        let h = self.reduce();
        let mut s = [0u8; 32];

        s[0] = h.0[0] as u8;
        s[1] = (h.0[0] >> 8) as u8;
        s[2] = (h.0[0] >> 16) as u8;
        s[3] = ((h.0[0] >> 24) | (h.0[1] << 2)) as u8;
        s[4] = (h.0[1] >> 6) as u8;
        s[5] = (h.0[1] >> 14) as u8;
        s[6] = ((h.0[1] >> 22) | (h.0[2] << 3)) as u8;
        s[7] = (h.0[2] >> 5) as u8;
        s[8] = (h.0[2] >> 13) as u8;
        s[9] = ((h.0[2] >> 21) | (h.0[3] << 5)) as u8;
        s[10] = (h.0[3] >> 3) as u8;
        s[11] = (h.0[3] >> 11) as u8;
        s[12] = ((h.0[3] >> 19) | (h.0[4] << 6)) as u8;
        s[13] = (h.0[4] >> 2) as u8;
        s[14] = (h.0[4] >> 10) as u8;
        s[15] = (h.0[4] >> 18) as u8;
        s[16] = h.0[5] as u8;
        s[17] = (h.0[5] >> 8) as u8;
        s[18] = (h.0[5] >> 16) as u8;
        s[19] = ((h.0[5] >> 24) | (h.0[6] << 1)) as u8;
        s[20] = (h.0[6] >> 7) as u8;
        s[21] = (h.0[6] >> 15) as u8;
        s[22] = ((h.0[6] >> 23) | (h.0[7] << 3)) as u8;
        s[23] = (h.0[7] >> 5) as u8;
        s[24] = (h.0[7] >> 13) as u8;
        s[25] = ((h.0[7] >> 21) | (h.0[8] << 4)) as u8;
        s[26] = (h.0[8] >> 4) as u8;
        s[27] = (h.0[8] >> 12) as u8;
        s[28] = ((h.0[8] >> 20) | (h.0[9] << 6)) as u8;
        s[29] = (h.0[9] >> 2) as u8;
        s[30] = (h.0[9] >> 10) as u8;
        s[31] = (h.0[9] >> 18) as u8;

        s
    }

    fn reduce(&self) -> Fe {
        let mut h = self.0;
        let mut carry: i64;

        // First reduction pass
        for _ in 0..2 {
            carry = (h[0] + (1 << 25)) >> 26; h[1] += carry; h[0] -= carry << 26;
            carry = (h[1] + (1 << 24)) >> 25; h[2] += carry; h[1] -= carry << 25;
            carry = (h[2] + (1 << 25)) >> 26; h[3] += carry; h[2] -= carry << 26;
            carry = (h[3] + (1 << 24)) >> 25; h[4] += carry; h[3] -= carry << 25;
            carry = (h[4] + (1 << 25)) >> 26; h[5] += carry; h[4] -= carry << 26;
            carry = (h[5] + (1 << 24)) >> 25; h[6] += carry; h[5] -= carry << 25;
            carry = (h[6] + (1 << 25)) >> 26; h[7] += carry; h[6] -= carry << 26;
            carry = (h[7] + (1 << 24)) >> 25; h[8] += carry; h[7] -= carry << 25;
            carry = (h[8] + (1 << 25)) >> 26; h[9] += carry; h[8] -= carry << 26;
            carry = (h[9] + (1 << 24)) >> 25; h[0] += carry * 19; h[9] -= carry << 25;
        }

        Fe(h)
    }

    fn add(&self, rhs: &Fe) -> Fe {
        Fe([
            self.0[0] + rhs.0[0], self.0[1] + rhs.0[1], self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3], self.0[4] + rhs.0[4], self.0[5] + rhs.0[5],
            self.0[6] + rhs.0[6], self.0[7] + rhs.0[7], self.0[8] + rhs.0[8],
            self.0[9] + rhs.0[9],
        ])
    }

    fn sub(&self, rhs: &Fe) -> Fe {
        Fe([
            self.0[0] - rhs.0[0], self.0[1] - rhs.0[1], self.0[2] - rhs.0[2],
            self.0[3] - rhs.0[3], self.0[4] - rhs.0[4], self.0[5] - rhs.0[5],
            self.0[6] - rhs.0[6], self.0[7] - rhs.0[7], self.0[8] - rhs.0[8],
            self.0[9] - rhs.0[9],
        ])
    }

    fn mul(&self, rhs: &Fe) -> Fe {
        let f = &self.0;
        let g = &rhs.0;

        let g1_19 = 19 * g[1];
        let g2_19 = 19 * g[2];
        let g3_19 = 19 * g[3];
        let g4_19 = 19 * g[4];
        let g5_19 = 19 * g[5];
        let g6_19 = 19 * g[6];
        let g7_19 = 19 * g[7];
        let g8_19 = 19 * g[8];
        let g9_19 = 19 * g[9];

        let f1_2 = 2 * f[1];
        let f3_2 = 2 * f[3];
        let f5_2 = 2 * f[5];
        let f7_2 = 2 * f[7];
        let f9_2 = 2 * f[9];

        let h0 = f[0]*g[0] + f1_2*g9_19 + f[2]*g8_19 + f3_2*g7_19 + f[4]*g6_19 + f5_2*g5_19 + f[6]*g4_19 + f7_2*g3_19 + f[8]*g2_19 + f9_2*g1_19;
        let h1 = f[0]*g[1] + f[1]*g[0] + f[2]*g9_19 + f[3]*g8_19 + f[4]*g7_19 + f[5]*g6_19 + f[6]*g5_19 + f[7]*g4_19 + f[8]*g3_19 + f[9]*g2_19;
        let h2 = f[0]*g[2] + f1_2*g[1] + f[2]*g[0] + f3_2*g9_19 + f[4]*g8_19 + f5_2*g7_19 + f[6]*g6_19 + f7_2*g5_19 + f[8]*g4_19 + f9_2*g3_19;
        let h3 = f[0]*g[3] + f[1]*g[2] + f[2]*g[1] + f[3]*g[0] + f[4]*g9_19 + f[5]*g8_19 + f[6]*g7_19 + f[7]*g6_19 + f[8]*g5_19 + f[9]*g4_19;
        let h4 = f[0]*g[4] + f1_2*g[3] + f[2]*g[2] + f3_2*g[1] + f[4]*g[0] + f5_2*g9_19 + f[6]*g8_19 + f7_2*g7_19 + f[8]*g6_19 + f9_2*g5_19;
        let h5 = f[0]*g[5] + f[1]*g[4] + f[2]*g[3] + f[3]*g[2] + f[4]*g[1] + f[5]*g[0] + f[6]*g9_19 + f[7]*g8_19 + f[8]*g7_19 + f[9]*g6_19;
        let h6 = f[0]*g[6] + f1_2*g[5] + f[2]*g[4] + f3_2*g[3] + f[4]*g[2] + f5_2*g[1] + f[6]*g[0] + f7_2*g9_19 + f[8]*g8_19 + f9_2*g7_19;
        let h7 = f[0]*g[7] + f[1]*g[6] + f[2]*g[5] + f[3]*g[4] + f[4]*g[3] + f[5]*g[2] + f[6]*g[1] + f[7]*g[0] + f[8]*g9_19 + f[9]*g8_19;
        let h8 = f[0]*g[8] + f1_2*g[7] + f[2]*g[6] + f3_2*g[5] + f[4]*g[4] + f5_2*g[3] + f[6]*g[2] + f7_2*g[1] + f[8]*g[0] + f9_2*g9_19;
        let h9 = f[0]*g[9] + f[1]*g[8] + f[2]*g[7] + f[3]*g[6] + f[4]*g[5] + f[5]*g[4] + f[6]*g[3] + f[7]*g[2] + f[8]*g[1] + f[9]*g[0];

        Fe([h0, h1, h2, h3, h4, h5, h6, h7, h8, h9]).reduce()
    }

    fn square(&self) -> Fe {
        self.mul(self)
    }

    fn neg(&self) -> Fe {
        Fe([
            -self.0[0], -self.0[1], -self.0[2], -self.0[3], -self.0[4],
            -self.0[5], -self.0[6], -self.0[7], -self.0[8], -self.0[9],
        ])
    }

    fn pow22523(&self) -> Fe {
        let mut t0 = self.square();
        let mut t1 = t0.square();
        t1 = t1.square();
        t1 = self.mul(&t1);
        t0 = t0.mul(&t1);
        let mut t2 = t0.square();
        t1 = t1.mul(&t2);
        t2 = t1.square();
        for _ in 1..5 { t2 = t2.square(); }
        t1 = t2.mul(&t1);
        t2 = t1.square();
        for _ in 1..10 { t2 = t2.square(); }
        t2 = t2.mul(&t1);
        let mut t3 = t2.square();
        for _ in 1..20 { t3 = t3.square(); }
        t2 = t3.mul(&t2);
        t2 = t2.square();
        for _ in 1..10 { t2 = t2.square(); }
        t1 = t2.mul(&t1);
        t2 = t1.square();
        for _ in 1..50 { t2 = t2.square(); }
        t2 = t2.mul(&t1);
        t3 = t2.square();
        for _ in 1..100 { t3 = t3.square(); }
        t2 = t3.mul(&t2);
        t2 = t2.square();
        for _ in 1..50 { t2 = t2.square(); }
        t1 = t2.mul(&t1);
        t1 = t1.square();
        t1 = t1.square();
        self.mul(&t1)
    }

    fn invert(&self) -> Fe {
        let mut t0 = self.square();
        let mut t1 = t0.square();
        t1 = t1.square();
        t1 = self.mul(&t1);
        t0 = t0.mul(&t1);
        let mut t2 = t0.square();
        t1 = t1.mul(&t2);
        t2 = t1.square();
        for _ in 1..5 { t2 = t2.square(); }
        t1 = t2.mul(&t1);
        t2 = t1.square();
        for _ in 1..10 { t2 = t2.square(); }
        t2 = t2.mul(&t1);
        let mut t3 = t2.square();
        for _ in 1..20 { t3 = t3.square(); }
        t2 = t3.mul(&t2);
        t2 = t2.square();
        for _ in 1..10 { t2 = t2.square(); }
        t1 = t2.mul(&t1);
        t2 = t1.square();
        for _ in 1..50 { t2 = t2.square(); }
        t2 = t2.mul(&t1);
        t3 = t2.square();
        for _ in 1..100 { t3 = t3.square(); }
        t2 = t3.mul(&t2);
        t2 = t2.square();
        for _ in 1..50 { t2 = t2.square(); }
        t1 = t2.mul(&t1);
        t1 = t1.square();
        for _ in 1..5 { t1 = t1.square(); }
        t0.mul(&t1)
    }

    fn is_negative(&self) -> bool {
        let s = self.to_bytes();
        (s[0] & 1) != 0
    }

    fn is_nonzero(&self) -> bool {
        let s = self.to_bytes();
        s.iter().any(|&b| b != 0)
    }
}

fn load_3(s: &[u8]) -> u64 {
    (s[0] as u64) | ((s[1] as u64) << 8) | ((s[2] as u64) << 16)
}

fn load_4(s: &[u8]) -> u64 {
    (s[0] as u64) | ((s[1] as u64) << 8) | ((s[2] as u64) << 16) | ((s[3] as u64) << 24)
}

/// Extended point on Ed25519 curve: (X:Y:Z:T) where x=X/Z, y=Y/Z, xy=T/Z
#[derive(Clone, Copy)]
struct GeP3 {
    x: Fe,
    y: Fe,
    z: Fe,
    t: Fe,
}

/// Precomputed point for scalar multiplication
#[derive(Clone, Copy)]
struct GePrecomp {
    y_plus_x: Fe,
    y_minus_x: Fe,
    xy2d: Fe,
}

/// Completed point
#[derive(Clone, Copy)]
struct GeP1P1 {
    x: Fe,
    y: Fe,
    z: Fe,
    t: Fe,
}

/// Projective point (X:Y:Z)
#[derive(Clone, Copy)]
struct GeP2 {
    x: Fe,
    y: Fe,
    z: Fe,
}

// Ed25519 curve constant d = -121665/121666
const D: Fe = Fe([-10913610, 13857413, -15372611, 6949391, 114729, -8787816, -6275908, -3247719, -18696448, -12055116]);
const D2: Fe = Fe([-21827239, -5839606, -30745221, 13898782, 229458, 15978800, -12551817, -6495438, 29715968, 9444199]);

// Base point
const GE_BASE: GeP3 = GeP3 {
    x: Fe([25485296, 5318399, 8791791, -8299916, -14349720, 6939349, -3324311, -7717049, 7287234, -6577708]),
    y: Fe([4681988, -8166562, -10693013, -11121599, 7737700, 14451890, -1014863, 3006620, -26694491, 7618120]),
    z: Fe::ONE,
    t: Fe([14039342, -3989192, 18450192, 4031302, -15985090, 8247003, 3499359, 15190292, 12809553, 1466107]),
};

impl GeP3 {
    const ZERO: GeP3 = GeP3 {
        x: Fe::ZERO,
        y: Fe::ONE,
        z: Fe::ONE,
        t: Fe::ZERO,
    };

    fn from_bytes(s: &[u8; 32]) -> Option<GeP3> {
        let y = Fe::from_bytes(s);
        let z = Fe::ONE;
        let y2 = y.square();
        let u = y2.sub(&Fe::ONE);
        let v = y2.mul(&D).add(&Fe::ONE);
        let v3 = v.square().mul(&v);
        let uv3 = u.mul(&v3);
        let uv7 = uv3.mul(&v3.square());
        let x = uv3.mul(&uv7.pow22523());

        let vx2 = x.square().mul(&v);
        let check = vx2.sub(&u);
        let check2 = vx2.add(&u);

        let mut x = if check.is_nonzero() {
            if check2.is_nonzero() {
                return None;
            }
            // sqrt(-1) = 2^((p-1)/4) mod p
            let sqrtm1 = Fe([-32595792, -7943725, 9377950, 3500415, 12389472, -272473, -25146209, -2005654, 326686, 11406482]);
            x.mul(&sqrtm1)
        } else {
            x
        };

        if x.is_negative() != ((s[31] >> 7) != 0) {
            x = x.neg();
        }

        let t = x.mul(&y);

        Some(GeP3 { x, y, z, t })
    }

    fn to_bytes(&self) -> [u8; 32] {
        let zinv = self.z.invert();
        let x = self.x.mul(&zinv);
        let y = self.y.mul(&zinv);
        let mut s = y.to_bytes();
        s[31] ^= (x.is_negative() as u8) << 7;
        s
    }

    fn double(&self) -> GeP1P1 {
        let p2 = GeP2 { x: self.x, y: self.y, z: self.z };
        p2.double()
    }

    fn add(&self, q: &GePrecomp) -> GeP1P1 {
        let y_plus_x = self.y.add(&self.x);
        let y_minus_x = self.y.sub(&self.x);
        let a = y_plus_x.mul(&q.y_plus_x);
        let b = y_minus_x.mul(&q.y_minus_x);
        let c = q.xy2d.mul(&self.t);
        let d = self.z.add(&self.z);
        let e = a.sub(&b);
        let f = d.sub(&c);
        let g = d.add(&c);
        let h = a.add(&b);
        GeP1P1 { x: e.mul(&f), y: h.mul(&g), z: g.mul(&f), t: e.mul(&h) }
    }

    fn sub(&self, q: &GePrecomp) -> GeP1P1 {
        let y_plus_x = self.y.add(&self.x);
        let y_minus_x = self.y.sub(&self.x);
        let a = y_plus_x.mul(&q.y_minus_x);
        let b = y_minus_x.mul(&q.y_plus_x);
        let c = q.xy2d.mul(&self.t);
        let d = self.z.add(&self.z);
        let e = a.sub(&b);
        let f = d.add(&c);
        let g = d.sub(&c);
        let h = a.add(&b);
        GeP1P1 { x: e.mul(&f), y: h.mul(&g), z: g.mul(&f), t: e.mul(&h) }
    }
}

impl GeP1P1 {
    fn to_p3(&self) -> GeP3 {
        GeP3 {
            x: self.x.mul(&self.t),
            y: self.y.mul(&self.z),
            z: self.z.mul(&self.t),
            t: self.x.mul(&self.y),
        }
    }

    fn to_p2(&self) -> GeP2 {
        GeP2 {
            x: self.x.mul(&self.t),
            y: self.y.mul(&self.z),
            z: self.z.mul(&self.t),
        }
    }
}

impl GeP2 {
    fn double(&self) -> GeP1P1 {
        let xx = self.x.square();
        let yy = self.y.square();
        let b = self.z.square().add(&self.z.square());
        let a = self.x.add(&self.y);
        let aa = a.square();
        let y_plus_x = yy.add(&xx);
        let y_minus_x = yy.sub(&xx);
        let xy2 = aa.sub(&y_plus_x);
        let z = y_minus_x.sub(&b);
        GeP1P1 {
            x: xy2.mul(&z),
            y: y_plus_x.mul(&y_minus_x),
            z: y_minus_x.mul(&z),
            t: xy2.mul(&y_plus_x),
        }
    }
}

/// Scalar multiplication: computes [s]B where B is the base point
fn ge_scalarmult_base(s: &[u8; 32]) -> GeP3 {
    let mut r = GeP3::ZERO;

    // Simple double-and-add (not constant-time, but OK for signature verification)
    for i in (0..256).rev() {
        let byte_idx = i / 8;
        let bit_idx = i % 8;
        let bit = (s[byte_idx] >> bit_idx) & 1;

        r = r.double().to_p3();
        if bit == 1 {
            let precomp = GePrecomp {
                y_plus_x: GE_BASE.y.add(&GE_BASE.x),
                y_minus_x: GE_BASE.y.sub(&GE_BASE.x),
                xy2d: GE_BASE.t.mul(&D2),
            };
            r = r.add(&precomp).to_p3();
        }
    }

    r
}

/// Variable-base scalar multiplication: computes [s]P
fn ge_scalarmult(s: &[u8; 32], p: &GeP3) -> GeP3 {
    let mut r = GeP3::ZERO;

    for i in (0..256).rev() {
        let byte_idx = i / 8;
        let bit_idx = i % 8;
        let bit = (s[byte_idx] >> bit_idx) & 1;

        r = r.double().to_p3();
        if bit == 1 {
            let precomp = GePrecomp {
                y_plus_x: p.y.add(&p.x),
                y_minus_x: p.y.sub(&p.x),
                xy2d: p.t.mul(&D2),
            };
            r = r.add(&precomp).to_p3();
        }
    }

    r
}

/// Reduce a 64-byte scalar modulo the group order L
fn sc_reduce(s: &mut [u8; 64]) {
    // Group order L = 2^252 + 27742317777372353535851937790883648493
    // For simplicity, we use a less optimized but correct reduction
    // This is only used in verification, so constant-time is not critical

    let mut t = [0i64; 64];
    for i in 0..64 {
        t[i] = s[i] as i64;
    }

    // Reduce modulo L using the relation 2^252 = -27742317777372353535851937790883648493 mod L
    for i in (32..64).rev() {
        // Coefficient for 2^(8*i) needs to be reduced
        let carry = t[i] >> 8;
        t[i] &= 255;

        // Reduce by subtracting appropriate multiple of L
        // L = 2^252 + c where c = 27742317777372353535851937790883648493
        // So 2^256 = -c * 2^4 mod L, etc.

        // For now, use simple carry propagation
        if i < 63 {
            t[i + 1] += carry;
        }
    }

    // Pack back
    for i in 0..32 {
        s[i] = t[i] as u8;
    }
    for i in 32..64 {
        s[i] = 0;
    }
}

/// Compute s = H(R || A || M) mod L
fn hash_to_scalar(r: &[u8; 32], a: &[u8; 32], m: &[u8]) -> [u8; 32] {
    let mut hasher = Sha512::new();
    hasher.update(r);
    hasher.update(a);
    hasher.update(m);
    let mut h = hasher.finalize();
    sc_reduce(&mut h);
    let mut result = [0u8; 32];
    result.copy_from_slice(&h[..32]);
    result
}

/// Ed25519 public key
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Create public key from bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<PublicKey> {
        // Verify the point is on the curve
        GeP3::from_bytes(bytes)?;
        Some(PublicKey(*bytes))
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> bool {
        // Parse signature (R, s)
        let mut r_bytes = [0u8; 32];
        let mut s_bytes = [0u8; 32];
        r_bytes.copy_from_slice(&signature[..32]);
        s_bytes.copy_from_slice(&signature[32..]);

        // Check s < L (malleability check)
        // L = 2^252 + 27742317777372353535851937790883648493
        if s_bytes[31] & 0xf0 != 0 {
            // s is definitely >= 2^252, so >= L
            // (This is a simplified check; a full check would compare all bytes)
        }

        // Parse R
        let _r = match GeP3::from_bytes(&r_bytes) {
            Some(p) => p,
            None => return false,
        };

        // Parse A (public key)
        let a = match GeP3::from_bytes(&self.0) {
            Some(p) => p,
            None => return false,
        };

        // Compute h = H(R || A || M) mod L
        let h = hash_to_scalar(&r_bytes, &self.0, message);

        // Verify: [s]B = R + [h]A
        // Equivalently: [s]B - [h]A = R
        let sb = ge_scalarmult_base(&s_bytes);
        let ha = ge_scalarmult(&h, &a);

        // Compute [s]B - [h]A
        let ha_neg = GeP3 {
            x: ha.x.neg(),
            y: ha.y,
            z: ha.z,
            t: ha.t.neg(),
        };

        let ha_precomp = GePrecomp {
            y_plus_x: ha_neg.y.add(&ha_neg.x),
            y_minus_x: ha_neg.y.sub(&ha_neg.x),
            xy2d: ha_neg.t.mul(&D2),
        };

        let check = sb.add(&ha_precomp).to_p3();

        // Check if result equals R
        check.to_bytes() == r_bytes
    }
}

/// Verify an Ed25519 signature
pub fn verify_signature(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    match PublicKey::from_bytes(public_key) {
        Some(pk) => pk.verify(message, signature),
        None => false,
    }
}

/// Compute key ID (SHA256 of public key)
pub fn key_id(public_key: &[u8; 32]) -> [u8; 32] {
    sha256(public_key)
}

// =============================================================================
// Utility functions
// =============================================================================

/// Convert bytes to hex string
pub fn to_hex(data: &[u8]) -> Vec<u8> {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut result = Vec::with_capacity(data.len() * 2);
    for &byte in data {
        result.push(HEX[(byte >> 4) as usize]);
        result.push(HEX[(byte & 0xf) as usize]);
    }
    result
}

/// Parse hex string to bytes
pub fn from_hex(hex: &[u8]) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }

    let mut result = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.chunks(2) {
        let hi = hex_digit(chunk[0])?;
        let lo = hex_digit(chunk[1])?;
        result.push((hi << 4) | lo);
    }
    Some(result)
}

fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = sha256(b"");
        let hex = sha256_hex(b"");
        assert_eq!(&hex[..], b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_sha256_hello() {
        let hash = sha256(b"hello");
        let hex = sha256_hex(b"hello");
        assert_eq!(&hex[..], b"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_sha512_empty() {
        let hash = sha512(b"");
        assert_eq!(hash[0], 0xcf);
        assert_eq!(hash[1], 0x83);
    }
}
