#![feature(lazy_cell, ptr_sub_ptr)]

use engage::gamedata::{skill::SkillData, unit::Unit, JobData};
use unity::{il2cpp::object::Array, prelude::*};
use skyline::patching::Patch;


#[unity::class("App", "InfoUtil_StatusSkill")]
pub struct StatusSkill {
    pub data: Option<&'static SkillData>,
    pub isactive: bool,
    pub category: i32,
}

#[unity::hook("App","Unit","LearnJobSkill")]
pub fn unit_learnjobskill(this: &Unit, job: &JobData, method_info: OptionalMethod) -> &'static SkillData {
    // Check if the learn skill exist and add it to EquipSkillPool
    if job.learn_skill.is_some() {
        let sid = job.learn_skill.unwrap();
        unsafe {
            unit_addtoequipskillpool(this, sid, None);
        }
    }    
    // Call the original function and return the SkillData
    call_original!(this, job, method_info)
}

#[unity::hook("App","InfoUtil","GetSkillListForUnitInfo")]
pub fn infoutil_getskilllistforunitinfo(unit: &Unit, isskillequip: bool, ispack: bool, size: i32 , method_info: OptionalMethod) -> &Array<Option<&'static StatusSkill>> {
    let statusskills = call_original!(unit, isskillequip, ispack, size, method_info);
    // The skill will always exist but not the SkillData, so we check that the JobSkill does
    if statusskills[1].is_some() {
        if statusskills[1].unwrap().data.is_some() {
            // Equip Slots are at 2 and 3
            for x in 2..4 as usize {
                if statusskills[x].is_some(){ 
                    // No need to disable if already disabled
                    if statusskills[x].unwrap().isactive {
                        let skill = statusskills[x].unwrap().data;
                        // Check if the equip skill has valid data
                        if skill.is_some() {
                            let skill = skill.unwrap().sid.get_string().unwrap_or("None".to_string());
                            let learnskill = statusskills[1].unwrap().data.unwrap().sid.get_string().unwrap_or("".to_string());
                            // Compare the SIDs of the JobSkill and EquipSkill
                            if skill == learnskill {
                                unsafe {
                                    // Set EquipSkill inactive if JobSkill is the same
                                    statusskill_setactive(statusskills[x].unwrap(), false, None);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    statusskills
}

// Function to set skill as active or inactive
#[skyline::from_offset(0x01fc7310)]
pub fn statusskill_setactive(this: &StatusSkill, value: bool, method_info: OptionalMethod);

// Function to add to EquipSkillPool, same is done in Inheritance
#[unity::from_offset("App", "Unit", "AddToEquipSkillPool")]
pub fn unit_addtoequipskillpool(this: &Unit, sid: &Il2CppString, method_info: OptionalMethod);

pub fn get_force(unit: &Unit) -> i32 {
    let force = unit.force;
    match force {
        Some(force) => force.force_type,
        // Putting at 1 so that if the force does not exist it will make the force check false.
        None => 1,
    }
}

#[skyline::main(name = "equipskl")]
pub fn main() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };


        let err_msg = format!(
            "JobSkill plugin has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );

        skyline::error::show_error(
            42069,
            "JobSkill plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));
    // Patch these address to nop parts of functions which are called if SP Cost of a skill = 0
    let addresses = [0x01a35fa4, 0x01a36f34, 0x01a36588, 0x01a38b68, 0x01a38b68, 0x01a35ec8, 0x01a391e8];
    for address in addresses {
        let patch = Patch::in_text(address).nop();
        if patch.is_ok() {
            patch.unwrap();
            println!("Patched address {:x} with NOP", address);
        }
    }
    skyline::install_hooks!(unit_learnjobskill, infoutil_getskilllistforunitinfo);
}


