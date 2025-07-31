slint::include_modules!();

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct Measurements {
    chest: f64,
    abdominal: f64,
    thigh: f64,
    triceps: f64,
    subscapular: f64,
    suprailiac: f64,
    midaxillary: f64,
}

impl Measurements {
    fn new() -> Self {
        Self {
            chest: 0.0,
            abdominal: 0.0,
            thigh: 0.0,
            triceps: 0.0,
            subscapular: 0.0,
            suprailiac: 0.0,
            midaxillary: 0.0,
        }
    }
    
    fn total(&self) -> f64 {
        self.chest + self.abdominal + self.thigh + self.triceps 
            + self.subscapular + self.suprailiac + self.midaxillary
    }
    
    fn set_measurement(&mut self, site: &str, value: f64) {
        match site {
            "chest" => self.chest = value,
            "abdominal" => self.abdominal = value,
            "thigh" => self.thigh = value,
            "triceps" => self.triceps = value,
            "subscapular" => self.subscapular = value,
            "suprailiac" => self.suprailiac = value,
            "midaxillary" => self.midaxillary = value,
            _ => {}
        }
    }
}

fn calculate_body_fat(total_measurement: f64, age: u32, is_male: bool) -> f64 {
    let body_density = if is_male {
        // Male Jackson & Pollock 7-site equation
        1.112 - 0.00043499 * total_measurement 
            + 0.00000055 * total_measurement.powi(2) 
            - 0.00028826 * (age as f64)
    } else {
        // Female Jackson & Pollock 7-site equation
        1.097 - 0.00046971 * total_measurement 
            + 0.00000056 * total_measurement.powi(2) 
            - 0.00012828 * (age as f64)
    };
    
    (495.0 / body_density) - 450.0
}

fn classify_body_fat_male(age: u32, bf: f64) -> &'static str {
    if bf < 5.0 {
        return "Extremely Lean (Below Essential Fat)";
    }
    
    let age_groups = [
        (20, 29, [(5.0, 13.8, "Excellent"), (13.9, 17.4, "Good"), (17.5, 20.4, "Average"), (20.5, 24.1, "Below Average"), (24.2, 100.0, "Poor")]),
        (30, 39, [(5.0, 14.9, "Excellent"), (15.0, 18.9, "Good"), (19.0, 21.4, "Average"), (21.5, 25.1, "Below Average"), (25.2, 100.0, "Poor")]),
        (40, 49, [(5.0, 16.9, "Excellent"), (17.0, 19.9, "Good"), (20.0, 22.4, "Average"), (22.5, 26.1, "Below Average"), (26.2, 100.0, "Poor")]),
        (50, 59, [(5.0, 18.9, "Excellent"), (19.0, 21.9, "Good"), (22.0, 24.4, "Average"), (24.5, 28.1, "Below Average"), (28.2, 100.0, "Poor")]),
        (60, 69, [(5.0, 20.9, "Excellent"), (21.0, 23.9, "Good"), (24.0, 26.4, "Average"), (26.5, 30.1, "Below Average"), (30.2, 100.0, "Poor")])
    ];
    
    for (lower_age, upper_age, ranges) in age_groups.iter() {
        if age >= *lower_age && age <= *upper_age {
            for (low, high, category) in ranges.iter() {
                if bf >= *low && bf <= *high {
                    return category;
                }
            }
        }
    }
    
    "Unclassified"
}

fn classify_body_fat_female(age: u32, bf: f64) -> &'static str {
    if bf < 10.0 {
        return "Extremely Lean (Below Essential Fat)";
    }
    
    let age_groups = [
        (20, 29, [(10.0, 18.0, "Excellent"), (19.0, 23.0, "Good"), (24.0, 29.0, "Average"), (30.0, 35.0, "Below Average"), (36.0, 100.0, "Poor")]),
        (30, 39, [(11.0, 19.0, "Excellent"), (20.0, 24.0, "Good"), (25.0, 30.0, "Average"), (31.0, 36.0, "Below Average"), (37.0, 100.0, "Poor")]),
        (40, 49, [(12.0, 20.0, "Excellent"), (21.0, 25.0, "Good"), (26.0, 31.0, "Average"), (32.0, 37.0, "Below Average"), (38.0, 100.0, "Poor")]),
        (50, 59, [(13.0, 21.0, "Excellent"), (22.0, 26.0, "Good"), (27.0, 32.0, "Average"), (33.0, 38.0, "Below Average"), (39.0, 100.0, "Poor")]),
        (60, 69, [(14.0, 22.0, "Excellent"), (23.0, 27.0, "Good"), (28.0, 33.0, "Average"), (34.0, 39.0, "Below Average"), (40.0, 100.0, "Poor")])
    ];
    
    for (lower_age, upper_age, ranges) in age_groups.iter() {
        if age >= *lower_age && age <= *upper_age {
            for (low, high, category) in ranges.iter() {
                if bf >= *low && bf <= *high {
                    return category;
                }
            }
        }
    }
    
    "Unclassified"
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = BodyFatCalculator::new()?;
    let ui_handle = ui.as_weak();
    
    // Store measurements in a shared state
    let measurements = Rc::new(RefCell::new(Measurements::new()));
    
    // Handle measurement updates
    ui.on_measurement_updated({
        let measurements = measurements.clone();
        move |site, value| {
            if let Ok(parsed_value) = value.parse::<f64>() {
                measurements.borrow_mut().set_measurement(&site, parsed_value);
                println!("Updated {} measurement: {}", site, parsed_value);
            }
        }
    });
    
    // Handle body fat calculation
    ui.on_calculate_body_fat({
        let ui_handle = ui_handle.clone();
        let measurements = measurements.clone();
        move || {
            let ui = ui_handle.upgrade().unwrap();
            
            // Get current measurements from UI (as fallback) and stored state
            let current_measurements = measurements.borrow().clone();
            let mut final_measurements = Measurements::new();
            let mut parse_errors = Vec::new();
            
            // Helper to get measurement from UI or stored state
            let get_measurement = |ui_value: slint::SharedString, stored_value: f64, site: &str| -> Result<f64, String> {
                // Prefer UI value if present, otherwise use stored value
                if !ui_value.is_empty() {
                    ui_value.parse::<f64>()
                        .map_err(|_| format!("{} measurement must be a valid number", site))
                } else if stored_value > 0.0 {
                    Ok(stored_value)
                } else {
                    Err(format!("{} measurement is required", site))
                }
            };
            
            // Get all measurements (UI takes precedence over stored state)
            match get_measurement(ui.get_chest_measurement(), current_measurements.chest, "Chest") {
                Ok(val) => final_measurements.chest = val,
                Err(e) => parse_errors.push(e),
            }
            
            match get_measurement(ui.get_abdominal_measurement(), current_measurements.abdominal, "Abdominal") {
                Ok(val) => final_measurements.abdominal = val,
                Err(e) => parse_errors.push(e),
            }
            
            match get_measurement(ui.get_thigh_measurement(), current_measurements.thigh, "Thigh") {
                Ok(val) => final_measurements.thigh = val,
                Err(e) => parse_errors.push(e),
            }
            
            match get_measurement(ui.get_triceps_measurement(), current_measurements.triceps, "Triceps") {
                Ok(val) => final_measurements.triceps = val,
                Err(e) => parse_errors.push(e),
            }
            
            match get_measurement(ui.get_subscapular_measurement(), current_measurements.subscapular, "Subscapular") {
                Ok(val) => final_measurements.subscapular = val,
                Err(e) => parse_errors.push(e),
            }
            
            match get_measurement(ui.get_suprailiac_measurement(), current_measurements.suprailiac, "Suprailiac") {
                Ok(val) => final_measurements.suprailiac = val,
                Err(e) => parse_errors.push(e),
            }
            
            match get_measurement(ui.get_midaxillary_measurement(), current_measurements.midaxillary, "Midaxillary") {
                Ok(val) => final_measurements.midaxillary = val,
                Err(e) => parse_errors.push(e),
            }
            
            // Parse age
            let age = match ui.get_age_input().parse::<u32>() {
                Ok(age) if age > 0 && age < 120 => age,
                _ => {
                    parse_errors.push("Age must be a valid number between 1 and 119".to_string());
                    0
                }
            };
            
            // Check for errors
            if !parse_errors.is_empty() {
                ui.set_result_text(format!("Errors: {}", parse_errors.join(", ")).into());
                ui.set_category_text("Please fix the errors above".into());
                ui.set_show_results(true);
                return;
            }
            
            // Calculate body fat
            let is_male = ui.get_selected_gender() == "Male";
            let total_measurement = final_measurements.total();
            let body_fat_percentage = calculate_body_fat(total_measurement, age, is_male);
            
            // Classify result
            let category = if is_male {
                classify_body_fat_male(age, body_fat_percentage)
            } else {
                classify_body_fat_female(age, body_fat_percentage)
            };
            
            // Update UI
            ui.set_result_text(format!("Body Fat Percentage: {:.2}%", body_fat_percentage).into());
            ui.set_category_text(format!("Category for age {} ({}): {}", 
                age, 
                if is_male { "Male" } else { "Female" }, 
                category
            ).into());
            ui.set_show_results(true);
            
            // Update stored measurements with final values
            *measurements.borrow_mut() = final_measurements;
        }
    });
    
    ui.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_body_fat_male() {
        let bf = calculate_body_fat(100.0, 30, true);
        assert!(bf > 0.0 && bf < 50.0); // Reasonable range
    }
    
    #[test]
    fn test_calculate_body_fat_female() {
        let bf = calculate_body_fat(100.0, 30, false);
        assert!(bf > 0.0 && bf < 50.0); // Reasonable range
    }
    
    #[test]
    fn test_measurements_total() {
        let mut measurements = Measurements::new();
        measurements.chest = 10.0;
        measurements.abdominal = 15.0;
        assert_eq!(measurements.total(), 25.0);
    }
}