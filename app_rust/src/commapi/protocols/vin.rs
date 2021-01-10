


#[derive(Debug, Clone)]
pub struct Vin {
    pub raw: String,
    pub year: u32,
    pub manufacture_location: String,
    pub manufacture_name: String
}

impl Vin {

    fn get_year(id: char) -> u32 {
        match id {
            _ => unimplemented!()
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
            _ => ("Unknown", "Unknown")
        };
        return (res.0.into(), res.1.into())
    }


    pub fn new(str: String) -> Option<Self> {
        return if str.len() != 17 {
            None
        } else {
            let info = Vin::get_wmi(&str[0..3]);
            Some(Self {
                raw: str,
                year: 2000,
                manufacture_location: info.0,
                manufacture_name: info.1
            })
        }
    }
}