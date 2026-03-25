const RESULTING_WIDTH = 1920;
const RESULTING_HEIGHT = 1080;


struct Params{
    screen_width: u32,
    screen_height: u32,
    block_length: u32,
    screen_width_difference: f32,
    screen_height_difference: f32,
    resulting_width: u32,
    resulting_height: u32
};


// A read-only storage buffer that stores and array of unsigned 32bit integers
@group(0) @binding(0) var<storage, read> input: array<u32>;

@group(0) @binding(1) var<uniform> params:Params;

// This storage buffer can be read from and written to
@group(0) @binding(2) var<storage, read_write> output1: array<u32>;

@group(0) @binding(3) var<storage, read_write> output2: array<u32>;

@group(0) @binding(4) var<storage, read_write> output3: array<u32>;

var<workgroup> shared_cr: array<u32,64>;
var<workgroup> shared_cb: array<u32,64>;

// Tells wgpu that this function is a valid compute pipeline entry_point
@compute
// Specifies the "dimension" of this work group
@workgroup_size(8,8)
fn main(
    // global_invocation_id specifies our position in the invocation grid
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>,
) {
    let index = global_invocation_id.x + global_invocation_id.y * RESULTING_WIDTH;
    let total = arrayLength(&input);

    // workgroup_size may not be a multiple of the array size so
    // we need to exit out a thread would index out of bounds.
    if (index >= total) {
        return;
    }

    var r_average:u32 = 0;
    var b_average:u32 = 0;
    var g_average:u32 = 0;

    var r:u32 = 0;
    var g:u32 = 0;
    var b:u32 = 0;

    let pos = f32(global_invocation_id.x) * params.screen_width_difference  + f32(global_invocation_id.y) * params.screen_height_difference * f32(params.screen_width);

    for(var i:u32 = 0; i < params.block_length;i++)
    {
        for(var j:u32 = 0; j < params.block_length;j++)
        {
            let index = u32(pos) + j + (i * params.screen_width);
            r_average += input[index] & 0xFF;
            g_average += (input[index] >> 8) & 0xFF;
            b_average += (input[index] >> 16) & 0xFF;
        }
    }

    r = r_average / (params.block_length * params.block_length);
    g = g_average / (params.block_length * params.block_length);
    b = b_average / (params.block_length * params.block_length);

    r_average = 0;
    g_average = 0;
    b_average = 0;

    let shared_pos = local_invocation_id.x + local_invocation_id.y * 8;

    output1[index] = convert_y(r,g,b);
    shared_cr[shared_pos] = convert_cr(r,g,b);
    shared_cb[shared_pos] = convert_cb(r,g,b);

    storageBarrier();

    if(local_invocation_id.x % 2 != 0 || local_invocation_id.y % 2 != 0)
    {
        return;
    }

    var cr_average:u32 = 0;
    var cb_average:u32 = 0;
    
    for(var i:u32 = 0; i < 2;i++)
    {
        for(var j:u32 = 0; j < 2;j++)
        {
            let pos = shared_pos + i + (j * 8);
            cr_average += shared_cr[pos];
            cb_average += shared_cb[pos];
        }
    }

    let farb_x = global_invocation_id.x / 2;
    let farb_y = global_invocation_id.y / 2;
    let farb_index = farb_x + farb_y * (params.resulting_width / 2); 

    output2[farb_index] = cr_average / 4;
    output3[farb_index] = cb_average / 4;
}


fn convert_y(r:u32, g:u32, b:u32) -> u32{
    let y = 0.299  * f32(r)  + 0.587 * f32(g) + 0.114 * f32(b);
    return u32(y) ;
}

fn convert_cb(r:u32, g:u32, b:u32) -> u32 {
    let cb = -0.168736 * f32(r) - 0.331264 * f32(g) + 0.5 * f32(b) + 128.0;
    return u32(cb);
}

fn convert_cr(r:u32, g:u32, b:u32) -> u32 {
    let cr = 0.5 * f32(r) - 0.418688 * f32(g) - 0.081312 * f32(b) + 128.0;
    return u32(cr);

}

