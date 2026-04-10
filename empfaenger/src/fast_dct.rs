use std::vec;
//https://stackoverflow.com/questions/29215879/how-can-i-generalize-the-quantization-matrix-in-jpeg-compression

pub fn dct_quant(vector: &mut [f32]){
    transform_horizontal(vector);
    transform_vertical(vector);
    dct_matrix(vector);
}

pub fn inverse_dct_quant(vector: &mut [f32])
{
    inverse_dct_matrix(vector);
    inverse_horizontal(vector);
    inverse_vertical(vector);
}

pub fn transform_horizontal(vector: &mut [f32]){
    // Algorithm by Arai, Agui, Nakajima, 1988. For details, see:
    // https://web.stanford.edu/class/ee398a/handouts/lectures/07-TransformCoding.pdf#page=30
    for i in (0..vector.len()).step_by(8)
    {
        let v0 = vector[0 + i] + vector[7 + i];
        let v1 = vector[1 + i] + vector[6 + i];
        let v2 = vector[2 + i] + vector[5 + i];
        let v3 = vector[3 + i] + vector[4 + i];
        let v4 = vector[3 + i] - vector[4 + i];
        let v5 = vector[2 + i] - vector[5 + i];
        let v6 = vector[1 + i] - vector[6 + i];
        let v7 = vector[0 + i] - vector[7 + i];
         
        let v8 = v0 + v3;
        let v9 = v1 + v2;
        let v10 = v1 - v2;
        let v11 = v0 - v3;
        let v12 = -v4 - v5;
        let v13 = (v5 + v6) * A[3];
        let v14 = v6 + v7;
        
        let v15 = v8 + v9;
        let v16 = v8 - v9;
        let v17 = (v10 + v11) * A[1];
        let v18 = (v12 + v14) * A[5];
        
        let v19 = -v12 * A[2] - v18;
        let v20 = v14 * A[4] - v18;
        
        let v21 = v17 + v11;
        let v22 = v11 - v17;
        let v23 = v13 + v7;
        let v24 = v7 - v13;
        
        let v25 = v19 + v24;
        let v26 = v23 + v20;
        let v27 = v23 - v20;
        let v28 = v24 - v19;
        
        vector[0 + i] = S[0] * v15;
        vector[1 + i] = S[1] * v26;
        vector[2 + i] = S[2] * v21;
        vector[3 + i] = S[3] * v28;
        vector[4 + i] = S[4] * v16;
        vector[5 + i] = S[5] * v25;
        vector[6 + i] = S[6] * v22;
        vector[7 + i] = S[7] * v27;
    }
}

pub fn transform_vertical(vector: &mut [f32]) {
    for col in 0..8 {
        let v0 = vector[col + 0*8] + vector[col + 7*8];
        let v1 = vector[col + 1*8] + vector[col + 6*8];
        let v2 = vector[col + 2*8] + vector[col + 5*8];
        let v3 = vector[col + 3*8] + vector[col + 4*8];
        let v4 = vector[col + 3*8] - vector[col + 4*8];
        let v5 = vector[col + 2*8] - vector[col + 5*8];
        let v6 = vector[col + 1*8] - vector[col + 6*8];
        let v7 = vector[col + 0*8] - vector[col + 7*8];

        let v8  = v0 + v3;
        let v9  = v1 + v2;
        let v10 = v1 - v2;
        let v11 = v0 - v3;
        let v12 = -v4 - v5;
        let v13 = (v5 + v6) * A[3];
        let v14 = v6 + v7;

        let v15 = v8 + v9;
        let v16 = v8 - v9;
        let v17 = (v10 + v11) * A[1];
        let v18 = (v12 + v14) * A[5];

        let v19 = -v12 * A[2] - v18;
        let v20 = v14 * A[4] - v18;

        let v21 = v17 + v11;
        let v22 = v11 - v17;
        let v23 = v13 + v7;
        let v24 = v7 - v13;

        let v25 = v19 + v24;
        let v26 = v23 + v20;
        let v27 = v23 - v20;
        let v28 = v24 - v19;

        vector[col + 0*8] = S[0] * v15;
        vector[col + 1*8] = S[1] * v26;
        vector[col + 2*8] = S[2] * v21;
        vector[col + 3*8] = S[3] * v28;
        vector[col + 4*8] = S[4] * v16;
        vector[col + 5*8] = S[5] * v25;
        vector[col + 6*8] = S[6] * v22;
        vector[col + 7*8] = S[7] * v27;
    }
}


/* 
 * Computes the scaled DCT type III on the given length-8 array in place.
 * The inverse of this function is transform(), except for rounding errors.
 */
pub fn inverse_transform(vector: &mut [f32]) {
    // A straightforward inverse of the forward algorithm
    let v15 = vector[0] / S[0];
    let v26 = vector[1] / S[1];
    let v21 = vector[2] / S[2];
    let v28 = vector[3] / S[3];
    let v16 = vector[4] / S[4];
    let v25 = vector[5] / S[5];
    let v22 = vector[6] / S[6];
    let v27 = vector[7] / S[7];
    
    let v19 = (v25 - v28) / 2.0;
    let v20 = (v26 - v27) / 2.0;
    let v23 = (v26 + v27) / 2.0;
    let v24 = (v25 + v28) / 2.0;
    
    let v7  = (v23 + v24) / 2.0;
    let v11 = (v21 + v22) / 2.0;
    let v13 = (v23 - v24) / 2.0;
    let v17 = (v21 - v22) / 2.0;
    
    let v8 = (v15 + v16) / 2.0;
    let v9 = (v15 - v16) / 2.0;
    
    let v18 = (v19 - v20) * A[5];  // Different from original
    let v12 = (v19 * A[4] - v18) / (A[2] * A[5] - A[2] * A[4] - A[4] * A[5]);
    let v14 = (v18 - v20 * A[2]) / (A[2] * A[5] - A[2] * A[4] - A[4] * A[5]);
    
    let v6 = v14 - v7;
    let v5 = v13 / A[3] - v6;
    let v4 = -v5 - v12;
    let v10 = v17 / A[1] - v11;
    
    let v0 = (v8 + v11) / 2.0;
    let v1 = (v9 + v10) / 2.0;
    let v2 = (v9 - v10) / 2.0;
    let v3 = (v8 - v11) / 2.0;
    
    vector[0] = (v0 + v7) / 2.0;
    vector[1] = (v1 + v6) / 2.0;
    vector[2] = (v2 + v5) / 2.0;
    vector[3] = (v3 + v4) / 2.0;
    vector[4] = (v3 - v4) / 2.0;
    vector[5] = (v2 - v5) / 2.0;
    vector[6] = (v1 - v6) / 2.0;
    vector[7] = (v0 - v7) / 2.0;
}


pub fn inverse_horizontal(vector: &mut [f32]){
    for i in (0..vector.len()).step_by(8){
        let v15 = vector[0 + i] / S[0];
        let v26 = vector[1 + i] / S[1];
        let v21 = vector[2 + i] / S[2];
        let v28 = vector[3 + i] / S[3];
        let v16 = vector[4 + i] / S[4];
        let v25 = vector[5 + i] / S[5];
        let v22 = vector[6 + i] / S[6];
        let v27 = vector[7 + i] / S[7];


        let v19 = (v25 - v28) / 2.0;
        let v20 = (v26 - v27) / 2.0;
        let v23 = (v26 + v27) / 2.0;
        let v24 = (v25 + v28) / 2.0;
        
        let v7  = (v23 + v24) / 2.0;
        let v11 = (v21 + v22) / 2.0;
        let v13 = (v23 - v24) / 2.0;
        let v17 = (v21 - v22) / 2.0;
        
        let v8 = (v15 + v16) / 2.0;
        let v9 = (v15 - v16) / 2.0;
        
        let v18 = (v19 - v20) * A[5];  // Different from original
        let v12 = (v19 * A[4] - v18) / (A[2] * A[5] - A[2] * A[4] - A[4] * A[5]);
        let v14 = (v18 - v20 * A[2]) / (A[2] * A[5] - A[2] * A[4] - A[4] * A[5]);
        
        let v6 = v14 - v7;
        let v5 = v13 / A[3] - v6;
        let v4 = -v5 - v12;
        let v10 = v17 / A[1] - v11;
        
        let v0 = (v8 + v11) / 2.0;
        let v1 = (v9 + v10) / 2.0;
        let v2 = (v9 - v10) / 2.0;
        let v3 = (v8 - v11) / 2.0;


        vector[0 + i] = (v0 + v7) / 2.0;
        vector[1 + i] = (v1 + v6) / 2.0;
        vector[2 + i] = (v2 + v5) / 2.0;
        vector[3 + i] = (v3 + v4) / 2.0;
        vector[4 + i] = (v3 - v4) / 2.0;
        vector[5 + i] = (v2 - v5) / 2.0;
        vector[6 + i] = (v1 - v6) / 2.0;
        vector[7 + i] = (v0 - v7) / 2.0;
    }
}


pub fn inverse_vertical(vector: &mut [f32]){
    for col in 0..8{
        let v15 = vector[8 * 0 + col] / S[0];
        let v26 = vector[8 * 1 + col] / S[1];
        let v21 = vector[8 * 2 + col] / S[2];
        let v28 = vector[8 * 3 + col] / S[3];
        let v16 = vector[8 * 4 + col] / S[4];
        let v25 = vector[8 * 5 + col] / S[5];
        let v22 = vector[8 * 6 + col] / S[6];
        let v27 = vector[8 * 7 + col] / S[7];


        let v19 = (v25 - v28) / 2.0;
        let v20 = (v26 - v27) / 2.0;
        let v23 = (v26 + v27) / 2.0;
        let v24 = (v25 + v28) / 2.0;
        
        let v7  = (v23 + v24) / 2.0;
        let v11 = (v21 + v22) / 2.0;
        let v13 = (v23 - v24) / 2.0;
        let v17 = (v21 - v22) / 2.0;
        
        let v8 = (v15 + v16) / 2.0;
        let v9 = (v15 - v16) / 2.0;
        
        let v18 = (v19 - v20) * A[5];  // Different from original
        let v12 = (v19 * A[4] - v18) / (A[2] * A[5] - A[2] * A[4] - A[4] * A[5]);
        let v14 = (v18 - v20 * A[2]) / (A[2] * A[5] - A[2] * A[4] - A[4] * A[5]);
        
        let v6 = v14 - v7;
        let v5 = v13 / A[3] - v6;
        let v4 = -v5 - v12;
        let v10 = v17 / A[1] - v11;
        
        let v0 = (v8 + v11) / 2.0;
        let v1 = (v9 + v10) / 2.0;
        let v2 = (v9 - v10) / 2.0;
        let v3 = (v8 - v11) / 2.0;


        vector[8 *  0 + col] = (v0 + v7) / 2.0;
        vector[8 *  1 + col] = (v1 + v6) / 2.0;
        vector[8 *  2 + col] = (v2 + v5) / 2.0;
        vector[8 *  3 + col] = (v3 + v4) / 2.0;
        vector[8 *  4 + col] = (v3 - v4) / 2.0;
        vector[8 *  5 + col] = (v2 - v5) / 2.0;
        vector[8 *  6 + col] = (v1 - v6) / 2.0;
        vector[8 *  7 + col] = (v0 - v7) / 2.0;
    }
}

const COMPRESSION_STRENGTH:f32 = 1.0;

pub fn dct_matrix(vector: &mut [f32])
{
    for i in 0..64{
        vector[i] = vector[i] / (MATRIX[i] * COMPRESSION_STRENGTH);
    }
}

pub fn inverse_dct_matrix(vector: &mut [f32])
{
    for i in 0..64{
        vector[i] = vector[i] * MATRIX[i] * COMPRESSION_STRENGTH;
    }
}


/*---- Tables of constants ----*/
const S: [f32; 8] = [
    0.353553390593273762200422,
    0.254897789552079584470970,
    0.270598050073098492199862,
    0.300672443467522640271861,
    0.353553390593273762200422,
    0.449988111568207852319255,
    0.653281482438188263928322,
    1.281457723870753089398043,
];

const A: [f32; 6] = [
    std::f32::NAN,
    0.707106781186547524400844,
    0.541196100146196984399723,
    0.707106781186547524400844,
    1.306562964876376527856643,
    0.382683432365089771728460,
];

const MATRIX: [f32; 64] = [
    16.0, 11.0, 10.0, 16.0, 24.0, 40.0, 51.0, 61.0,
    12.0, 12.0, 14.0, 19.0, 26.0, 58.0, 60.0, 55.0,
    14.0, 13.0, 16.0, 24.0, 40.0, 57.0, 69.0, 56.0,
    14.0, 17.0, 22.0, 29.0, 51.0, 87.0, 80.0, 62.0,
    18.0, 22.0, 37.0, 56.0, 68.0, 109.0, 103.0, 77.0,
    24.0, 35.0, 55.0, 64.0, 81.0, 104.0, 113.0, 92.0,
    49.0, 64.0, 78.0, 87.0, 103.0, 121.0, 120.0, 101.0,
    72.0, 92.0, 95.0, 98.0, 112.0, 100.0, 103.0, 99.0,
];