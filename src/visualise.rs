use chrono::Local;
use plotters::prelude::*;

pub fn plot_adsb_frame(buf: Vec<i32>) {
    // Plot magnitude
    let filename = format!("magnitude_plot_{}.png", Local::now());
    let root = BitMapBackend::new(&filename, (1024, 768)).into_drawing_area();

    root.fill(&WHITE);

    // Compute magnitudes and store them as i32 values
    let min_val = *buf.iter().min().unwrap_or(&0);
    let max_val = *buf.iter().max().unwrap_or(&1);

    let mut chart = ChartBuilder::on(&root)
    .caption("Magnitude of SDR Samples", ("sans-serif", 30))
    .margin(20)
    .x_label_area_size(30)
    .y_label_area_size(40)
    .build_cartesian_2d(0..buf.len() as usize, min_val..max_val).unwrap();

    chart.configure_mesh().draw();

    chart
    .draw_series(LineSeries::new(
        buf.iter().enumerate().map(|(i, &m)| (i, m)),
        &BLUE,
    )).unwrap()
    .label("Magnitude")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    root.present();
    println!("Plot saved to {}", filename);
}

/// Print out a preamble to allow for checking of it
/// 
fn print_preamble(buf: Vec<i32>) {
    for val in buf {
            print!(" {:^5} ", val);
    }

    print!("\n");

    for val in 0..16 {
        print!(" {:^5} ", val);
    }
    print!("\n");


} 

fn print_preamble_graph(buf: Vec<i32>) {
    let mut changed_buf: Vec<i32> = Vec::new();
    let max_val = buf.iter().max().unwrap();
    
    // for i in 0..16 {
    //     changed_buf = 3

    // }
    print!("\u{2581}");
}