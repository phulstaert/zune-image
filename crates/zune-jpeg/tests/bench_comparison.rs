use std::io::Cursor;
use std::time::Instant;
use zune_jpeg::JpegDecoder;

#[test]
fn run_integrated_benchmarks() {
    let baseline_img: &[u8] = include_bytes!("../../../test-images/jpeg/benchmarks/speed_bench.jpg");
    let dnl_img: &[u8] = include_bytes!("../../../test-images/jpeg/benchmarks/speed_bench_dnl.jpg");

    println!("\n=======================================================");
    println!("             ZUNE-JPEG INTEGRATED BENCHMARKS           ");
    println!("=======================================================");

    // Determine dimensions dynamically
    let (width, height, channels) = {
        let mut decoder = JpegDecoder::new(Cursor::new(baseline_img));
        decoder.decode_headers().unwrap();
        let info = decoder.info().unwrap();
        (info.width as usize, info.height as usize, 3)
    };
    println!("Image dimensions: {}x{} ({} channels)", width, height, channels);

    // Warmup
    {
        let mut decoder = JpegDecoder::new(Cursor::new(baseline_img));
        let _ = decoder.decode().unwrap();
    }

    // Number of runs for benchmarking (lower for high-res images to keep tests fast)
    const RUNS: usize = 20;

    // 1. BASELINE (Full image)
    let t_baseline = bench_run("1. Baseline (Full Frame)", RUNS, || {
        let mut decoder = JpegDecoder::new(Cursor::new(baseline_img));
        let pixels = decoder.decode().unwrap();
        assert_eq!(pixels.len(), width * height * channels);
    });

    // 2. DNL (Define Number of Lines parsing)
    let t_dnl = bench_run("2. DNL Parsing (Full Frame)", RUNS, || {
        let mut decoder = JpegDecoder::new(Cursor::new(dnl_img));
        let pixels = decoder.decode().unwrap();
        assert_eq!(pixels.len(), width * height * channels);
    });

    // 3. ROI (Region of Interest Cropping only)
    // Crop a central region: x: 1000, y: 1000, w: 1920, h: 1080 (1080p subset)
    let crop_x = 1000;
    let crop_y = 1000;
    let crop_w = 1920;
    let crop_h = 1080;
    let t_roi = bench_run("3. ROI Crop (1920x1080 region)", RUNS, || {
        let mut decoder = JpegDecoder::new(Cursor::new(baseline_img));
        decoder.set_cropping_region(crop_x, crop_y, crop_w, crop_h);
        let pixels = decoder.decode().unwrap();
        assert_eq!(pixels.len(), crop_w * crop_h * channels);
    });

    // 4. DOWN-SCALING / RESIZING only (factor 2, 4, 8)
    let t_scale2 = bench_run("4a. Downscale 1/2 (Factor 2)", RUNS, || {
        let mut decoder = JpegDecoder::new(Cursor::new(baseline_img));
        decoder.set_downscale_factor(2);
        let pixels = decoder.decode().unwrap();
        let w = width / 2;
        let h = height / 2;
        assert_eq!(pixels.len(), w * h * channels);
    });
    
    let t_scale4 = bench_run("4b. Downscale 1/4 (Factor 4)", RUNS, || {
        let mut decoder = JpegDecoder::new(Cursor::new(baseline_img));
        decoder.set_downscale_factor(4);
        let pixels = decoder.decode().unwrap();
        let w = width / 4;
        let h = height / 4;
        assert_eq!(pixels.len(), w * h * channels);
    });

    let t_scale8 = bench_run("4c. Downscale 1/8 (Factor 8)", RUNS, || {
        let mut decoder = JpegDecoder::new(Cursor::new(baseline_img));
        decoder.set_downscale_factor(8);
        let pixels = decoder.decode().unwrap();
        let w = width / 8;
        let h = height / 8;
        assert_eq!(pixels.len(), w * h * channels);
    });

    // 5. COMBINED (DNL + Crop + Downscale)
    let t_combined = bench_run("5. Combined (DNL + ROI Crop 1920x1080 + Downscale Factor 2)", RUNS, || {
        let mut decoder = JpegDecoder::new(Cursor::new(dnl_img));
        decoder.set_cropping_region(crop_x, crop_y, crop_w, crop_h);
        decoder.set_downscale_factor(2);
        let pixels = decoder.decode().unwrap();
        assert_eq!(pixels.len(), (crop_w / 2) * (crop_h / 2) * channels);
    });

    println!("\n---------------- PERFORMANCE RATIOS ----------------");
    println!("Baseline decode speed:  {:.3} ms / run (1.00x)", t_baseline);
    println!("DNL decode speed:       {:.3} ms / run ({:.2}x)", t_dnl, t_baseline / t_dnl);
    println!("ROI Crop (1920x1080):   {:.3} ms / run ({:.2}x speedup)", t_roi, t_baseline / t_roi);
    println!("Downscale Factor 2:     {:.3} ms / run ({:.2}x speedup)", t_scale2, t_baseline / t_scale2);
    println!("Downscale Factor 4:     {:.3} ms / run ({:.2}x speedup)", t_scale4, t_baseline / t_scale4);
    println!("Downscale Factor 8:     {:.3} ms / run ({:.2}x speedup)", t_scale8, t_baseline / t_scale8);
    println!("Combined (DNL+ROI+DS2): {:.3} ms / run ({:.2}x speedup)", t_combined, t_baseline / t_combined);
    println!("=======================================================\n");
}

fn bench_run<F: FnMut()>(name: &str, runs: usize, mut f: F) -> f64 {
    let start = Instant::now();
    for _ in 0..runs {
        f();
    }
    let elapsed = start.elapsed().as_secs_f64();
    let ms_per_run = (elapsed / runs as f64) * 1000.0;
    println!("- {}: {:.3} ms (total: {:.3}s)", name, ms_per_run, elapsed);
    ms_per_run
}
