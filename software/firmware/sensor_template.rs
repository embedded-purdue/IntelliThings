// Raw data from the sensor
#[derive(Debug)]
struct sensor_response([u8; 6]);

impl sensor_response {
    fn convert(&self) -> formatted_data {}
}

struct formatted_data {
    // Actual values we want to display/send
}

impl fmt::Debug for formatted_data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}");
    }
}

pub async fn read() -> formatted_data {
    // Pin addresses or any constants needed for this sensor
    const ADDRESS: u8 = 0x44;

    let mut response: sensor_response;

    // actually do reading

    // Debugging
    println!("{}", response);

    response.convert()
}
