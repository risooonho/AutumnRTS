
use crate::sim_components::active_ability_comp::ActiveAbilityComp;
use crate::sim_components::sim_unit_base_components::PositionComp;
use crate::sim_systems::input_systems::plc_building;
use crate::sim_ecs::SimState;
use crate::common::*;
use crate::sim_abilities_list::Ability;
use crate::sim_fix_math::*;
use hecs::*;

pub fn use_ability(
	sim: &mut SimState,
	entity: UId,
	target: ObjTarget,
	ability: &mut Ability){

	match ability {
		Ability::BuildSimpleStructure => build_simple_structure(sim, entity, target),
		Ability::GenericAbility{pw_cost: _pw,
			cooldown_end_at: mut cd, 
			range: _r, 
			damage: dmg} => generic_ability(sim, &mut cd, &dmg),
		Ability::Mundane => (),
	}
}


/// Tnh even I myself am impressed how ugly this function turned out.
/// But at least it works (hopefully)
pub fn sys_abilities(sim: &mut SimState) {

	let inbox = &mut sim.inbox;

    let (abil_msg, rest): (Vec<RenderMessage>, Vec<RenderMessage>) =
        inbox.iter().partition(|&msg| match msg {
        // inbox.clone().iter().partition(|&msg| match msg {
            RenderMessage::UseAbility(..) => true,
            _ => false,
        });

    *inbox = rest;

    for i in 0..abil_msg.len(){
    	match abil_msg[i] {
    		RenderMessage::UseAbility(id, abil_id, trg) => {

    			println!("Gonna use ability {:?}", abil_msg[i]);
    			// Use entity Id to get which ability to use:
    			if abil_id >= N_ABILITY_CAP {
    				continue;
    			}
    			let abil_comp = sim
    				.ecs
    				.get::<ActiveAbilityComp>(Entity::from_bits(id));
    			let abil_comp = abil_comp.unwrap();

    			let mut abil = abil_comp.get_ability(abil_id);
    			drop(abil_comp);
    			use_ability(sim, id, trg, &mut abil);
    		},
    		_ => (),
    	};
    }

}

fn build_simple_structure(
	sim: &mut SimState,
	id: UId,
	target: ObjTarget,
	) {

	if let ObjTarget::Position(pos) = target {
		// Now find the rounded position (tile) of caster
		// And rounded position (tile) of target
		// If tiles are adjacent then spawn_structure.

		if !sim.map.within(pos){
			return;
		}

		let pos0 = pos.round();

		let builder_pos = sim.ecs.get::<PositionComp>(Entity::from_bits(id)).unwrap();

		let pos1 = builder_pos.get_pos().round();
		drop(builder_pos); // fuck you borrow checker.

		// is adjacent?
		if pos0.dist(&pos1) < FixF::from_num(2.0){
			// now actually spawn a structure.
			if sim.map.tile_from_pos(pos0).blocks_path(){
                return;
            }

            if sim.map.map_mem.get_blocked().contains(&pos0.round()){
                return;
            }
            let mut new_structure = plc_building(pos0, &mut sim.id_counter);
            let e = sim.ecs.spawn(new_structure.build());

            sim.map.add_structure(vec![pos0]);

            let msg = EngineMessage::StructurePosTmp(e.to_bits(), pos0.round());
            sim.send_batch.push(msg);
		}
	}
}

fn generic_ability(
	_sim: &mut SimState, 
	cooldown_end_at: &mut TickNum, 
	damage: &i32) {


	println!("Casting ability! Deals {:?} damage!", damage); 
	*cooldown_end_at += 30; // 30 ticks cooldown.
}



#[cfg(test)]
mod ability_test {

	#[test]
	fn structure_ability(){
		use crate::sim_gameloop::*;
		use crate::common::*;
		use std::time::Duration;
		use crate::sim_fix_math::*;

		let (sim_handle, rend_msg) = start_loop(1, 30);

		rend_msg.send(vec![RenderMessage::Spawn(Pos::from_num(2, 7))]);

		// wait 0.5 seconds:
		::std::thread::sleep(Duration::from_secs_f32(0.5));

		// send messages:
		let msg = vec![
			RenderMessage::UseAbility(0,
				0, 
				ObjTarget::Position(
					Pos::from_num(3,7))),
		];

		rend_msg.send(msg);

		::std::thread::sleep(Duration::from_secs_f32(1.0));

		let inbox = rend_msg.rec();

		// for i in 0..inbox.len(){
		// 	match inbox[i] {
		// 		EngineMessage::StructurePosTmp(..) => {
		// 			println!("{:?}", inbox[i]);
		// 		}
		// 		_ => (),
		// 	}
		// }

		//println!("{:?}", inbox);

		// end game loop
        rend_msg.send(vec![RenderMessage::Break]);
        sim_handle.join().unwrap();
        
        println!("test ended");

	}
}