// use ndarray::*;
use std::collections::VecDeque;
use std::f64::consts::PI;

use crate::common_values;
use crate::gamestates::game_state::GameState;
use crate::gamestates::physics_object::PhysicsObject;
use crate::gamestates::player_data::PlayerData;

use super::obs_builder::ObsBuilder;

/// Matrix's observation builder, holds a stack of previous ball positions and shows the stack in the observation
pub struct AdvancedObsPadderStacker {
    team_size: usize,
    pos_std: f64,
    ang_std: f64,
    // expanding: bool,
    default_ball: [[f64; 3]; 3],
    stack_size: usize,
    ball_stack: Vec<VecDeque<[[f64; 3]; 3]>>
}

impl AdvancedObsPadderStacker {
    // pub fn new(team_size: Option<usize>, expanding: Option<bool>, stack_size: Option<usize>) -> Self {
    pub fn new(team_size: Option<usize>, stack_size: Option<usize>) -> Self {
        let team_size = match team_size {
            Some(team_size) => team_size,
            None => 3
        };
        // let expanding = match expanding {
        //     Some(expanding) => expanding,
        //     None => false
        // };
        let stack_size = match stack_size {
            Some(stack_size) => stack_size,
            None => 15
        };

        let mut advobsps = AdvancedObsPadderStacker {
            team_size: team_size,
            pos_std: 2300.,
            ang_std: PI,
            // expanding: expanding,
            default_ball: [[0.; 3]; 3],
            stack_size: stack_size,
            ball_stack: Vec::<VecDeque<[[f64; 3]; 3]>>::with_capacity(8)
        };
        for _i in 0..8 {
            advobsps.blank_stack()
        }
        return advobsps
    }

    fn blank_stack(&mut self) {
        let mut default_deque = VecDeque::with_capacity(self.stack_size+1);
        for _i in 0..self.stack_size {
            default_deque.push_front(self.default_ball.clone());
        }
        self.ball_stack.push(default_deque)
        // for _ in 0..self.stack_size {
        //     self.ball_stack[index].push_front(self.default_ball.clone())
        // }
    }

    fn add_ball_to_stack(&mut self, mut pos_std: [f64; 3], mut lin_std: [f64; 3], mut ang_std: [f64; 3], index: usize) {
        // to match Python functionality unfortunately (using extendleft from deque), this whole part needs to be redone for clarity
        pos_std.reverse();
        lin_std.reverse();
        ang_std.reverse();

        self.ball_stack[index].push_front([pos_std, lin_std, ang_std]);
        self.ball_stack[index].truncate(self.stack_size);
    }

    fn _add_dummy(obs: &mut Vec<f64>) {
        obs.extend([0.; 31].iter());
    }

    fn _add_player_to_obs(&self, obs: &mut Vec<f64>, car: &PlayerData, ball: &PhysicsObject, inverted: bool, player: Option<&PhysicsObject>) -> PhysicsObject {
        let mut player_car: PhysicsObject;
        if inverted {
            player_car = car.inverted_car_data;
        } else {
            player_car = car.car_data;
        }

        let mut rel_pos = ball.position - player_car.position;
        rel_pos = rel_pos.divide_by_var(self.pos_std);
        let mut rel_vel = ball.linear_velocity - player_car.linear_velocity;
        rel_vel = rel_vel.divide_by_var(self.pos_std);
        
        obs.extend(rel_pos.into_array().iter());
        obs.extend(rel_vel.into_array().iter());
        obs.extend(player_car.position.divide_by_var(self.pos_std).into_array().iter());
        obs.extend(player_car.forward().iter());
        obs.extend(player_car.up().iter());
        obs.extend(player_car.linear_velocity.divide_by_var(self.pos_std).into_array().iter());
        obs.extend(player_car.angular_velocity.divide_by_var(self.ang_std).into_array().iter());
        obs.extend(vec![car.boost_amount, car.on_ground as i32 as f64, car.has_flip as i32 as f64, car.is_demoed as i32 as f64]);

        match player {
            Some(player) => {
                obs.extend((player_car.position - player.position).divide_by_var(self.pos_std).into_array().iter());
                obs.extend((player_car.linear_velocity - player.linear_velocity).divide_by_var(self.pos_std).into_array().iter());
            }
            None => ()
        };

        return player_car
    }
}

impl ObsBuilder for AdvancedObsPadderStacker {
    fn reset(&mut self, _initial_state: &GameState) {
        
    }

    fn get_obs_space(&mut self) -> Vec<usize> {
        vec![276]
    }

    fn build_obs(&mut self, player: &PlayerData, state: &GameState, previous_action: &Vec<f64>) -> Vec<f64> {
        let inverted: bool;
        let ball: &PhysicsObject;
        let pads: [f64; 34];
        if player.team_num == common_values::ORANGE_TEAM {
           inverted = true;
           ball = &state.inverted_ball;
           pads = state.inverted_boost_pads; 
        } else {
            inverted = false;
            ball = &state.ball;
            pads = state.inverted_boost_pads;
        }

        let pos = &ball.position;
        let lin = &ball.linear_velocity;
        let ang = &ball.angular_velocity;

        // let pos_std = vec_div_variable(pos, &self.pos_std);
        // let lin_std = vec_div_variable(lin, &self.pos_std);
        // let ang_std = vec_div_variable(ang, &self.ang_std);
        let pos_std = pos.divide_by_var(self.pos_std);
        let lin_std = lin.divide_by_var(self.pos_std);
        let ang_std = ang.divide_by_var(self.ang_std);

        let mut obs = Vec::<f64>::with_capacity(276);

        obs.extend(pos_std.into_array().iter());
        obs.extend(lin_std.into_array().iter());
        obs.extend(ang_std.into_array().iter());
        obs.extend(previous_action.iter());
        obs.extend(pads.iter());

        // self.add_ball_to_stack(pos_std, lin_std, ang_std, player.car_id as usize);

        let ball_stack = &self.ball_stack[player.car_id as usize];
        for ball_vec in ball_stack {
            let pos_std_stack = &ball_vec[0];
            let lin_std_stack = &ball_vec[1];
            let ang_std_stack = &ball_vec[2];
            obs.extend(ang_std_stack.iter());
            obs.extend(lin_std_stack.iter());
            obs.extend(pos_std_stack.iter());
        }

        // need to do this better later, this is inefficient
        self.add_ball_to_stack(pos_std.into_array(), lin_std.into_array(), ang_std.into_array(), player.car_id as usize);

        let player_car = self._add_player_to_obs(&mut obs, &player, &ball, inverted, None);

        let mut ally_count = 0;
        let mut enemy_count = 0;

        for other in &state.players {
            if other.car_id == player.car_id {
                continue;
            }
            
            if other.team_num == player.team_num {
                ally_count += 1;
                if ally_count > self.team_size - 1 {
                    continue;
                }
            } else {
                enemy_count += 1;
                if enemy_count > self.team_size {
                    continue;
                }
            }

            self._add_player_to_obs(&mut obs, &other, ball, inverted, Some(&player_car));
        }

        while ally_count < self.team_size - 1 {
            AdvancedObsPadderStacker::_add_dummy(&mut obs);
            ally_count += 1;
        }
        while enemy_count < self.team_size {
            AdvancedObsPadderStacker::_add_dummy(&mut obs);
            enemy_count += 1;
        }

        return obs
    }
}