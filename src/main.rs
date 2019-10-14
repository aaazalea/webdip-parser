extern crate nom;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::opt,
    multi::many1,
    sequence::{delimited, separated_pair, terminated, tuple as nomtuple},
    IResult,
};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

impl fmt::Display for Game<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Reverse because WebDiplomacy shows results backwards
        for round in self.rounds.iter().rev() {
            writeln!(f, "##############################")?;
            writeln!(f, "# Diplomacy, {} {}", round.season, round.year)?;
            for pwr_round in &round.power_rounds {
                match &pwr_round.diplomacy {
                    Some(phase) => writeln!(f, "{}", phase)?,
                    None => (),
                }
            }
            writeln!(f, "##############################")?;
            writeln!(f, "# Retreats, {} {}", round.season, round.year)?;
            for pwr_round in &round.power_rounds {
                match &pwr_round.retreat {
                    Some(phase) => write!(f, "{}", phase)?,
                    None => (),
                }
            }
            if round.season == "Autumn" {
                writeln!(f, "\n##############################")?;
                writeln!(f, "# Builds, Winter {}", round.year)?;
                for pwr_round in &round.power_rounds {
                    match &pwr_round.build {
                        Some(phase) => write!(f, "{}", phase)?,
                        None => (),
                    }
                }
            }
            writeln!(f, "\n\n")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Game<'a> {
    pub rounds: Vec<Round<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Round<'a> {
    pub season: &'a str,
    pub year: &'a str,
    pub power_rounds: Vec<PowerRound<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PowerRound<'a> {
    pub power: &'a str,
    pub diplomacy: Option<Phase<'a>>,
    pub retreat: Option<Phase<'a>>,
    pub build: Option<Phase<'a>>,
}
impl fmt::Display for Phase<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for m in &self.moves {
            writeln!(f, "{}", m)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Phase<'a> {
    pub moves: Vec<Move<'a>>,
}

impl fmt::Display for Move<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.action {
            Action::Hold => write!(f, "{} hold", self.unit),
            Action::Build => write!(f, "build {}", self.unit),
            Action::Disband => write!(f, "{} disbands", self.unit),
            Action::Destroy => write!(f, "remove {}", self.unit),
            Action::Move { dst, manner } => write!(
                f,
                "{} -> {}{}",
                self.unit,
                dst,
                match manner {
                    TravelManner::Convoy => " by convoy",
                    _ => "",
                }
            ),
            Action::Retreat { dst } => write!(f, "{} -> {}", self.unit, dst),
            Action::SupportMove { src, dst } => write!(f, "{} S {} -> {}", self.unit, src, dst),
            Action::Convoy { src, dst } => write!(f, "{} convoys {} -> {}", self.unit, src, dst),
            Action::SupportHold { loc } => write!(f, "{} S {} hold", self.unit, loc),
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct Move<'a> {
    pub unit: Unit<'a>,
    pub action: Action<'a>,
}
#[derive(Debug, PartialEq)]
pub enum Action<'a> {
    Move {
        dst: Location<'a>,
        manner: TravelManner,
    },
    SupportMove {
        src: Location<'a>,
        dst: Location<'a>,
    },
    SupportHold {
        loc: Location<'a>,
    },
    Hold,
    Retreat {
        dst: Location<'a>,
    },
    Build,
    Disband,
    Destroy,
    Convoy {
        src: Location<'a>,
        dst: Location<'a>,
    },
}
#[derive(Debug, PartialEq)]
pub enum TravelManner {
    Convoy,
    Land,
}

#[derive(Debug, PartialEq)]
pub enum UnitType {
    Fleet,
    Army,
    UnspecifiedUnit,
}

impl fmt::Display for Unit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.unit_type {
            UnitType::Fleet => write!(f, "F {}", self.loc),
            UnitType::Army => write!(f, "A {}", self.loc),
            UnitType::UnspecifiedUnit => write!(f, "{}", self.loc),
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct Unit<'a> {
    pub unit_type: UnitType,
    pub loc: Location<'a>,
}

impl fmt::Display for Location<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.coast {
            // This is dumb but I can't figure out how to do it concisely less dumb. Coast should always be nonzero length.
            Some(coast) => write!(
                f,
                "{}/{}C",
                self.territory,
                match coast.chars().next() {
                    Some(c) => c,
                    None => 'C',
                }
            ),
            None => write!(f, "{}", self.territory),
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct Location<'a> {
    pub territory: &'a str,
    pub coast: Option<&'a str>,
}

fn parse_army(input: &str) -> IResult<&str, UnitType> {
    let (input, _) = tag("army at ")(input)?;
    Ok((input, UnitType::Army))
}
fn parse_fleet(input: &str) -> IResult<&str, UnitType> {
    let (input, _) = tag("fleet at ")(input)?;
    Ok((input, UnitType::Fleet))
}
fn parse_unit_unspecified(input: &str) -> IResult<&str, UnitType> {
    let (input, _) = tag("unit at ")(input)?;
    Ok((input, UnitType::UnspecifiedUnit))
}
fn parse_location(input: &str) -> IResult<&str, Location> {
    let (input, territory) = alt((
        alt((
            tag("Ankara"),
            tag("Belgium"),
            tag("Berlin"),
            tag("Brest"),
            tag("Budapest"),
            tag("Bulgaria"),
            tag("Constantinople"),
            tag("Denmark"),
            tag("Edinburgh"),
            tag("Greece"),
            tag("Holland"),
            tag("Kiel"),
            tag("Liverpool"),
            tag("London"),
            tag("Marseilles"),
            tag("Moscow"),
            tag("Munich"),
            tag("Naples"),
            tag("Norway"),
        )),
        alt((
            tag("Paris"),
            tag("Portugal"),
            tag("Rome"),
            tag("Rumania"),
            tag("St. Petersburg"),
            tag("Serbia"),
            tag("Sevastopol"),
            tag("Smyrna"),
            tag("Spain"),
            tag("Sweden"),
            tag("Trieste"),
            tag("Tunis"),
            tag("Venice"),
            tag("Vienna"),
            tag("Warsaw"),
            tag("Clyde"),
            tag("Yorkshire"),
            tag("Wales"),
        )),
        alt((
            tag("Picardy"),
            tag("Gascony"),
            tag("Burgundy"),
            tag("North Africa"),
            tag("Ruhr"),
            tag("Prussia"),
            tag("Silesia"),
            tag("Piedmont"),
            tag("Tuscany"),
            tag("Apulia"),
            tag("Tyrolia"),
            tag("Galicia"),
            tag("Bohemia"),
            tag("Finland"),
            tag("Livonia"),
            tag("Ukraine"),
        )),
        alt((
            tag("Albania"),
            tag("Armenia"),
            tag("Syria"),
            tag("North Atlantic Ocean"),
            tag("Mid-Atlantic Ocean"),
            tag("Norwegian Sea"),
            tag("North Sea"),
            tag("English Channel"),
            tag("Irish Sea"),
            tag("Heligoland Blight"),
            tag("Skagerrak"),
            tag("Baltic Sea"),
            tag("Gulf of Bothnia"),
            tag("Barents Sea"),
            tag("Western Mediterranean"),
        )),
        alt((
            tag("Gulf of Lyons"),
            tag("Tyrrhenian Sea"),
            tag("Ionian Sea"),
            tag("Adriatic Sea"),
            tag("Aegean Sea"),
            tag("Eastern Mediterranean"),
            tag("Black Sea"),
        )),
    ))(input)?;
    let (input, coast) = opt(delimited(
        tag(" ("),
        alt((tag("East"), tag("West"), tag("North"), tag("South"))),
        tag(" Coast)"),
    ))(input)?;
    Ok((input, Location { territory, coast }))
}
fn parse_unit(input: &str) -> IResult<&str, Unit> {
    let (input, (unit_type, loc)) = nomtuple((
        alt((parse_army, parse_fleet, parse_unit_unspecified)),
        parse_location,
    ))(input)?;
    Ok((input, Unit { unit_type, loc }))
}

fn build_order(input: &str) -> IResult<&str, Move> {
    let (input, unit) = delimited(tag("Build "), parse_unit, tag("."))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::Build,
        },
    ))
}
fn destroy_order(input: &str) -> IResult<&str, Move> {
    let (input, unit) = delimited(tag("Destroy the "), parse_unit, tag("."))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::Destroy,
        },
    ))
}
fn hold_order(input: &str) -> IResult<&str, Move> {
    let (input, unit) = delimited(tag("The "), parse_unit, tag(" hold."))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::Hold,
        },
    ))
}
fn parse_travel_manner(input: &str) -> IResult<&str, TravelManner> {
    let (input, convoy) = opt(tag(" via convoy"))(input)?;
    match convoy {
        None => Ok((input, TravelManner::Land)),
        Some(_) => Ok((input, TravelManner::Convoy)),
    }
}
fn move_order(input: &str) -> IResult<&str, Move> {
    let (input, (unit, dst, manner)) = nomtuple((
        delimited(tag("The "), parse_unit, tag(" move to ")),
        parse_location,
        terminated(parse_travel_manner, tag(".")),
    ))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::Move { dst, manner },
        },
    ))
}
fn support_move_order(input: &str) -> IResult<&str, Move> {
    let (input, (unit, dst, src)) = nomtuple((
        delimited(tag("The "), parse_unit, tag(" support move to ")),
        parse_location,
        delimited(tag(" from "), parse_location, tag(".")),
    ))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::SupportMove { src, dst },
        },
    ))
}
fn convoy_order(input: &str) -> IResult<&str, Move> {
    let (input, (unit, dst, src)) = nomtuple((
        delimited(tag("The "), parse_unit, tag(" convoy to ")),
        parse_location,
        delimited(tag(" from "), parse_location, tag(".")),
    ))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::Convoy { src, dst },
        },
    ))
}
fn support_hold_order(input: &str) -> IResult<&str, Move> {
    let (input, (unit, loc)) = nomtuple((
        delimited(tag("The "), parse_unit, tag(" support hold to ")),
        terminated(parse_location, tag(".")),
    ))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::SupportHold { loc },
        },
    ))
}
fn retreat_order(input: &str) -> IResult<&str, Move> {
    let (input, (unit, dst)) = nomtuple((
        delimited(tag("The "), parse_unit, tag(" retreat to ")),
        terminated(parse_location, tag(".")),
    ))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::Retreat { dst },
        },
    ))
}
fn disband_order(input: &str) -> IResult<&str, Move> {
    let (input, unit) = delimited(tag("The "), parse_unit, tag(" disband."))(input)?;
    Ok((
        input,
        Move {
            unit,
            action: Action::Disband,
        },
    ))
}
fn parse_order(input: &str) -> IResult<&str, Move> {
    let (input, mv) = terminated(
        alt((
            build_order,
            destroy_order,
            hold_order,
            move_order,
            support_move_order,
            support_hold_order,
            retreat_order,
            disband_order,
            convoy_order,
        )),
        nomtuple((opt(tag(" (fail)")), opt(tag(" (dislodged)")))),
    )(input)?;
    Ok((input, mv))
}
fn parse_phase(input: &str) -> IResult<&str, Phase> {
    let (input, moves) = many1(delimited(multispace0, parse_order, multispace0))(input)?;
    Ok((input, Phase { moves }))
}
fn parse_power_round(input: &str) -> IResult<&str, PowerRound> {
    let (input, power) = delimited(
        multispace0,
        alt((
            tag("Austria"),
            tag("England"),
            tag("France"),
            tag("Germany"),
            tag("Italy"),
            tag("Russia"),
            tag("Turkey"),
        )),
        tag(":"),
    )(input)?;
    let (input, diplomacy) = opt(delimited(
        delimited(multispace0, tag("Diplomacy"), multispace0),
        parse_phase,
        multispace0,
    ))(input)?;
    let (input, retreat) = opt(delimited(
        delimited(multispace0, tag("Retreats"), multispace0),
        parse_phase,
        multispace0,
    ))(input)?;
    let (input, build) = opt(delimited(
        delimited(multispace0, tag("Unit-placement"), multispace0),
        parse_phase,
        multispace0,
    ))(input)?;
    Ok((
        input,
        PowerRound {
            power,
            diplomacy,
            retreat,
            build,
        },
    ))
}
fn parse_round(input: &str) -> IResult<&str, Round> {
    let (input, (season, year)) = delimited(
        multispace0,
        separated_pair(
            alt((tag("Autumn"), tag("Spring"))),
            tag(", "),
            terminated(digit1, tag(" Large map:")),
        ),
        multispace0,
    )(input)?;
    let (input, power_rounds) =
        many1(delimited(multispace0, parse_power_round, multispace0))(input)?;
    Ok((
        input,
        Round {
            season,
            year,
            power_rounds,
        },
    ))
}
fn parse_game(input: &str) -> IResult<&str, Game> {
    let (input, rounds) = many1(delimited(multispace0, parse_round, multispace0))(input)?;
    Ok((input, Game { rounds }))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unit() {
        assert_eq!(
            parse_unit("fleet at Gulf of Lyons"),
            Ok((
                "",
                Unit {
                    unit_type: UnitType::Fleet,
                    loc: Location {
                        territory: "Gulf of Lyons",
                        coast: None
                    }
                }
            ))
        );
        assert_eq!(
            parse_unit("unit at Livonia"),
            Ok((
                "",
                Unit {
                    unit_type: UnitType::UnspecifiedUnit,
                    loc: Location {
                        territory: "Livonia",
                        coast: None
                    }
                }
            ))
        );
    }

    #[test]
    fn test_parse_order() {
        assert_eq!(
            parse_order("The army at Tyrolia move to Trieste."),
            Ok((
                "",
                Move {
                    unit: Unit {
                        unit_type: UnitType::Army,
                        loc: Location {
                            territory: "Tyrolia",
                            coast: None
                        }
                    },
                    action: Action::Move {
                        dst: Location {
                            territory: "Trieste",
                            coast: None
                        },
                        manner: TravelManner::Land
                    }
                }
            ))
        )
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("data.txt")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let res = parse_game(&contents);
    match res {
        Ok((input, game)) => {
            println!("Game: {}", game);
            assert_eq!(input, "");
        }
        Err(err) => println!("Error {:?}", err),
    }
    Ok(())
}
