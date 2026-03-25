use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::monitor::Monitor;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};

mod fastDct;
mod capture;
mod sdl_window;

impl GraphicsCaptureApiHandler for capture::Capture {

    // The type of flags used to get the values from the settings.
    type Flags = (mpsc::SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>, (u32,u32));

    // The type of error that can be returned from `CaptureControl` and `start`
    // functions.
    type Error = Box<dyn std::error::Error + Send + Sync>;

    // Function that will be called to create a new instance. The flags can be
    // passed from settings.
    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        //println!("Created with Flags: {}", ctx.flags);        

        let (sender, (width, height)) = ctx.flags;
        
        capture::Capture::new(width,height,sender)
    }

    // Called every time a new frame is available.
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {

        self.width  = frame.width();
        self.height = frame.height();
        let start = Instant::now();

        let mut data = frame.buffer()?;




        let raw: &mut [u8] = data.as_nopadding_buffer()?;



        //self.convert_rgb_yuv_shader(&raw);
        self.convert_rgbto_yuv_threaded(&raw);

        let mut blocks:Vec<u8> = self.linear_block_fast(&self.ycbcr.0);
        //self.linear_to_block_cb_cr(&self.ycbcr.1,&self.ycbcr.2);
       // self.dct_transformation();
        self.fast_dct(&mut blocks);
        //speichert letztes bild um vergleich zu ermöglchen sollte wahrscheinlich eher mit ownership gemacht werden todo();

        
        self.counter += 1;

        //sendet an anderen thread das Bild
        self.send_ycrcb();
        let end = start.elapsed().as_millis();

        if self.start.elapsed().as_secs() != self.second_last{
            self.second_last = self.start.elapsed().as_secs();
            println!("FPS: {}", self.counter);
            println!("Berechnung: {}ms", end);
            self.counter = 0;
        }
        Ok(())
    }


  

    // Optional handler called when the capture item (usually a window) is closed.
    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture session ended");

        Ok(())
    }
}

fn main(){
    let (tx,rx)  = mpsc::sync_channel::<(Vec<u8>,Vec<u8>,Vec<u8>)>(1);
// Gets the primary monitor, refer to the docs for other capture items.
let primary_monitor = Monitor::primary().expect("There is no primary monitor");

    println!("{:?}",primary_monitor.width() );
    println!("{:?}",primary_monitor.height() );

    let res = (primary_monitor.width().unwrap(), primary_monitor.height().unwrap());

    let flags = (tx, res);

let settings = Settings::new(
    // Item to capture
    primary_monitor,
    // Capture cursor settings
    CursorCaptureSettings::Default,
    // Draw border settings
    DrawBorderSettings::Default,
    // Secondary window settings, if you want to include secondary windows in the capture
    SecondaryWindowSettings::Default,
    // Minimum update interval, if you want to change the frame rate limit (default is 60 FPS or 16.67 ms)
    MinimumUpdateIntervalSettings::Custom(Duration::from_millis(5)),
    // Dirty region settings,
    DirtyRegionSettings::ReportOnly,
    // The desired color format for the captured frame.
    ColorFormat::Rgba8,
    // Additional flags for the capture settings that will be passed to the user-defined `new` function.
    flags,
);


thread::spawn(move || {
// Starts the capture and takes control of the current thread.
// The errors from the handler trait will end up here.
capture::Capture::start(settings).expect("Screen capture failed");
});

sdl_window::start_window(rx);

}