#[derive(Debug, Clone)]
pub struct Vin {
    pub raw: String,
    pub year: u32,
    pub manufacture_location: String,
    pub manufacture_name: String,
}

impl Vin {
    fn get_year(id: char) -> u32 {
        match id {
            'Y' => 2000,
            '1' => 2001,
            '2' => 2002,
            '3' => 2003,
            '4' => 2004,
            '5' => 2005,
            '6' => 2006,
            '7' => 2007,
            '8' => 2008,
            '9' => 2009,
            'A' => 2010,
            'B' => 2011,
            'C' => 2012,
            'D' => 2013,
            'E' => 2014,
            'F' => 2015,
            'G' => 2016,
            'H' => 2017,
            'J' => 2018,
            'K' => 2019,
            'L' => 2020,
            'M' => 2021,
            'N' => 2022,
            'P' => 2023,
            'R' => 2024,
            'S' => 2025,
            'T' => 2026,
            'V' => 2027,
            'W' => 2028,
            'X' => 2029,
            _ => 0,
        }
    }

    fn get_wmi(id: &str) -> (String, String) {
        let res = match id {
            "AAV" => ("South Africa", "Volkswagen"),
            "ADM" => ("South Africa", "General Motors of South Africa"),
            "ADN" => ("South Africa", "Nissan South Africa (Pty) Ltd"),
            "AHT" => ("South Africa", "Toyota"),
            "AFA" => ("South Africa", "Ford"),
            "BF9" => ("Kenya", "KIBO Motorcycles"),
            "CL9" => ("Tunisia", "Wallyscar"),
            "DA1" | "DA4" => ("Egypt", "Arab American Vehicles Company"),

            "JHL" | "JHM" => ("Japan", "Honda"),
            "JL5" => ("Japan", "Mitsubishi FUSO Truck & Bus Corp"),
            "JM0" => ("Japan", "Mazda for Oceania export"),
            "JM1" => ("Japan", "Mazda"),
            "JMB" => ("Japan", "Mitsubishi"),
            "JM6" => ("Japan", "Mazda"),
            z if &z[0..2] == "JN" => ("Japan", "Nissan"),
            z if &z[0..2] == "JS" => ("Japan", "Suzuki"),
            z if &z[0..2] == "JT" => ("Japan", "Toyota/Lexus"),
            z if &z[0..2] == "JY" => ("Japan", "Yamaha"),

            "WDB" => ("Germany", "Mercedes-Benz"),
            "WDC" | "WDD" | "WMX" => ("Germany", "Daimler AG"),
            _ => ("Unknown", "Unknown"),
        };
        (res.0.into(), res.1.into())
    }

    pub fn new(str: String) -> Option<Self> {
        return if str.len() != 17 {
            None
        } else {
            let info = Vin::get_wmi(&str[0..3]);
            let y = Vin::get_year(str.as_str().chars().nth(9).unwrap());
            Some(Self {
                raw: str,
                year: y,
                manufacture_location: info.0,
                manufacture_name: info.1,
            })
        };
    }
}
