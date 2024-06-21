use crate::TemperatureEntry;

/// This struct represents the temperature measures that have been sent to the web client
/// It will be a LIFO list
pub struct MeasuresManager {
    measures: Vec<TemperatureEntry>,
}

impl MeasuresManager {
    /// Returns an empty MeasuresManager
    pub fn new() -> MeasuresManager {
        MeasuresManager {
            measures: Vec::<TemperatureEntry>::new(),
        }
    }

    /// Add a measure at the begining of the list
    /// # Arguments
    ///
    /// * `measure` - An int
    ///
    pub fn add_new_measure(&mut self, measure: TemperatureEntry) {
        self.measures.insert(0, measure);
    }

    /// Returns all the measures list
    /// # Arguments    
    ///
    pub fn get_all_measures(&self) -> Vec<TemperatureEntry> {
        self.measures.clone()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::managers::measuresmanager::MeasuresManager;

//     #[test]
//     fn test_add_new_measure() {
//         let mut manager = MeasuresManager::new();
//         manager.add_new_measure(23);
//         assert_eq!(manager.get_last_measure_added().unwrap(), 23_u16);
//     }

//     #[test]
//     fn test_get_last_measure_added() {
//         let mut manager = MeasuresManager::new();
//         manager.add_new_measure(23);
//         manager.add_new_measure(50);
//         manager.add_new_measure(16);
//         assert_eq!(manager.get_last_measure_added().unwrap(), 16_u16);
//     }

//     #[test]
//     fn test_get_all_measures() {
//         let mut manager = MeasuresManager::new();
//         manager.add_new_measure(23);
//         manager.add_new_measure(50);
//         manager.add_new_measure(16);
//         manager.add_new_measure(128);

//         let my_vec = manager.get_all_measures();

//         assert_eq!(my_vec[0], 128);
//         assert_eq!(my_vec[1], 16);
//         assert_eq!(my_vec[2], 50);
//         assert_eq!(my_vec[3], 23);
//     }
// }
