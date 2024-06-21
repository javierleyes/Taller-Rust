use crate::temperature::temperature_entry::TemperatureEntry;

pub fn build_list(temperatures: &[TemperatureEntry]) -> String {
    let html = "<table><tr><th>Measured at</th><th>Temperature value</th></tr>{data}</table>";

    let data: Vec<String> = temperatures
        .iter()
        .map(|t| format!("<tr><td>{:#?}</td><td>{}</td></tr>", t.measured_at, t.value))
        .collect();
    html.replace("{data}", &data.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temperature::temperature_entry::TemperatureEntry;
    use std::time::SystemTime;

    #[test]
    fn test_build_empty_table() {
        let temperatures: Vec<TemperatureEntry> = Vec::new();

        let sut = build_list(&temperatures);
        assert_eq!(
            sut,
            "<table><tr><th>Measured at</th><th>Temperature value</th></tr></table>"
        );
    }

    #[test]
    fn test_build_table_with_values() {
        let temperatures = vec![
            TemperatureEntry {
                measured_at: SystemTime::now(),
                value: 30.0,
            },
            TemperatureEntry {
                measured_at: SystemTime::now(),
                value: 31.0,
            },
        ];

        let sut: String = build_list(&temperatures);
        assert!(sut.contains("30"));
        assert!(sut.contains("31"));
    }
}
