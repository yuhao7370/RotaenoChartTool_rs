mod chart;
use chart::chart::Chart;

fn main() {
    simple_logger::init().unwrap();
    let chart: Chart = Chart::load_chart_from_official("chart.txt").unwrap();

    // let chart: chart::chart::Chart = chart::chart::Chart::load_chart_from_json("jkbd_converted.json").unwrap();
    
    let _ =chart.export_to_json("converted.json");

    // let _ = chart.export_to_txt("jkbd.txt");

    // i from 0 to 100000
    // print!("{}", chart.version)
    // for i in 55560..=56000 {
    //     let degree = chart.find_degree_by_time(i as f32);
    //     println!("time: {}, degree: {}", i, degree); 
    //     // sleep(std::time::Duration::from_millis(10));
    // }

    // for i in 57900..=57950 {
    //     let degree = chart.find_degree_by_time(i as f32);
    //     println!("time: {}, degree: {}", i, degree); 
    //     // sleep(std::time::Duration::from_millis(10));
    // }
}

    
