use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub enum Rhythm {
    Quarter,
    Eighth,
    TripletEighth,
    TripletQuarterEighth,
    TripletEighthQuarter,
    Sixteenth,
    EighthSixteenths,
    SixteenEights,
}

//TODO change to floating points ?
impl Rhythm {
    pub fn make_intervals(&self, bpm: u64, beats_per_measure: usize) -> Vec<u64> {
        match &self {
            Rhythm::Quarter => {
                vec![60_000_000_000 / bpm; beats_per_measure]
            }
            Rhythm::Sixteenth => {
                vec![60_000_000_000 / bpm / 4; beats_per_measure * 4]
            }
            Rhythm::Eighth => {
                vec![60_000_000_000 / bpm / 2; beats_per_measure * 2]
            }
            _ => {
                vec![]
            }
        }
    }

    pub fn make_duration(&self, bpm: u64) -> u64 {
        match &self {
            Rhythm::Quarter => 60_000_000_000 / bpm,
            Rhythm::Sixteenth => 60_000_000_000 / bpm / 4,
            Rhythm::Eighth => 60_000_000_000 / bpm / 2,
            _ => 0,
        }
    }
}

impl FromStr for Rhythm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Quarter,
        // Eighth,
        // TripletEighth,
        // TripletQuarterEighth,
        // TripletEigthQuarter,
        // Sixteenth,
        // EigthSixteenths,
        // SixteenEights,

        match s {
            "quarters" => Ok(Rhythm::Quarter),
            "eights" => Ok(Rhythm::Eighth),
            "triplet_eights" => Ok(Rhythm::TripletEighth),
            "triplet_quarter_eights" => Ok(Rhythm::TripletQuarterEighth),
            "triplet_eighth_quarters" => Ok(Rhythm::TripletEighthQuarter),
            "sixteenths" => Ok(Rhythm::Sixteenth),
            "eighth_sixteenths" => Ok(Rhythm::EighthSixteenths),
            "sixteen_eights" => Ok(Rhythm::SixteenEights),
            _ => Err(()),
        }
    }
}
