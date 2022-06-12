use crate::defs::Structure;

pub fn structure_info(structure: &Structure) -> String {
    match structure {
        Structure::None | Structure::Occupied { .. } => unreachable!(),
        Structure::Branch => {
            t!("branch")
        }
        Structure::Core => {
            t!("core")
        }
        _ => todo!(),
    }
}
