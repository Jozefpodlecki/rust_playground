const BLOCK_SIZE: usize = 64;
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const ROUNDS: usize = 20;

#[repr(align(16))]
struct ChaChaState {
    state: [u32; 16],
}

impl ChaChaState {
    fn new(key: &[u8; KEY_SIZE], nonce: &[u8; NONCE_SIZE]) -> Self {
        let mut state = [0u32; 16];
        
        state[0] = 0x61707865;
        state[1] = 0x3320646E;
        state[2] = 0x79622D32;
        state[3] = 0x6B206574;
        
        for i in 0..8 {
            state[4 + i] = u32::from_le_bytes([
                key[i * 4],
                key[i * 4 + 1],
                key[i * 4 + 2],
                key[i * 4 + 3],
            ]);
        }
        
        state[12] = 0;
        state[13] = 0;
        state[14] = u32::from_le_bytes([
            nonce[0], nonce[1], nonce[2], nonce[3]
        ]);
        state[15] = u32::from_le_bytes([
            nonce[4], nonce[5], nonce[6], nonce[7]
        ]);
        
        ChaChaState { state }
    }
    
    fn quarter_round(&mut self, a: usize, b: usize, c: usize, d: usize) {
        self.state[a] = self.state[a].wrapping_add(self.state[b]);
        self.state[d] ^= self.state[a];
        self.state[d] = self.state[d].rotate_left(16);
        
        self.state[c] = self.state[c].wrapping_add(self.state[d]);
        self.state[b] ^= self.state[c];
        self.state[b] = self.state[b].rotate_left(12);
        
        self.state[a] = self.state[a].wrapping_add(self.state[b]);
        self.state[d] ^= self.state[a];
        self.state[d] = self.state[d].rotate_left(8);
        
        self.state[c] = self.state[c].wrapping_add(self.state[d]);
        self.state[b] ^= self.state[c];
        self.state[b] = self.state[b].rotate_left(7);
    }
    
    fn inner_block(&mut self) {
        for _ in 0..ROUNDS / 2 {
            self.quarter_round(0, 4, 8, 12);
            self.quarter_round(1, 5, 9, 13);
            self.quarter_round(2, 6, 10, 14);
            self.quarter_round(3, 7, 11, 15);
            self.quarter_round(0, 5, 10, 15);
            self.quarter_round(1, 6, 11, 12);
            self.quarter_round(2, 7, 8, 13);
            self.quarter_round(3, 4, 9, 14);
        }
    }
    
    fn next_block(&mut self) -> [u8; BLOCK_SIZE] {
        let mut working = ChaChaState { state: self.state };
        working.inner_block();
        
        for i in 0..16 {
            working.state[i] = working.state[i].wrapping_add(self.state[i]);
        }
        
        self.state[12] = self.state[12].wrapping_add(1);
        if self.state[12] == 0 {
            self.state[13] = self.state[13].wrapping_add(1);
        }
        
        let mut output = [0u8; BLOCK_SIZE];
        for i in 0..16 {
            let bytes = working.state[i].to_le_bytes();
            output[i * 4] = bytes[0];
            output[i * 4 + 1] = bytes[1];
            output[i * 4 + 2] = bytes[2];
            output[i * 4 + 3] = bytes[3];
        }
        output
    }
}

pub struct ChaChaRng {
    state: ChaChaState,
    buffer: [u8; BLOCK_SIZE],
    index: usize,
}

impl ChaChaRng {
    pub fn from_seed(seed: &[u8; KEY_SIZE]) -> Self {
        let nonce = [0u8; NONCE_SIZE];
        let state = ChaChaState::new(seed, &nonce);
        let mut rng = ChaChaRng {
            state,
            buffer: [0u8; BLOCK_SIZE],
            index: BLOCK_SIZE,
        };
        rng.refill();
        rng
    }
    
    fn refill(&mut self) {
        self.buffer = self.state.next_block();
        self.index = 0;
    }
    
    pub fn next_u32(&mut self) -> u32 {
        if self.index + 4 > BLOCK_SIZE {
            self.refill();
        }
        
        let bytes: [u8; 4] = self.buffer[self.index..self.index + 4].try_into().unwrap();
        self.index += 4;
        u32::from_le_bytes(bytes)
    }
    
    pub fn next_u64(&mut self) -> u64 {
        if self.index + 8 > BLOCK_SIZE {
            self.refill();
        }
        
        let bytes: [u8; 8] = self.buffer[self.index..self.index + 8].try_into().unwrap();
        self.index += 8;
        u64::from_le_bytes(bytes)
    }
    
    pub fn fill_bytes(&mut self, buffer: &mut [u8]) {
        let mut remaining = buffer.len();
        let mut offset = 0;
        
        while remaining > 0 {
            if self.index >= BLOCK_SIZE {
                self.refill();
            }
            
            let available = BLOCK_SIZE - self.index;
            let to_copy = if remaining < available { remaining } else { available };
            
            buffer[offset..offset + to_copy].copy_from_slice(&self.buffer[self.index..self.index + to_copy]);
            self.index += to_copy;
            offset += to_copy;
            remaining -= to_copy;
        }
    }
}