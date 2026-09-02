// For creating lookup table to be used in `assembly_sources/stepper_music`
// Generates version 1 compatible assembly code, does not take advantage of timers

static MOVE_CYCLES: u32 = 24;
static HALT_CYCLES: u32 = 8;
static FREQ_LOOP_MOVES: u32 = 22;
static FREQ_LOOP_MOVES_DIRECT_LOOKUP: u32 = 22;
static OUTER_LOOP_MOVES: u32 = 27;

pub fn generate_lookup_table(clock: u32) -> String {
	let t_loop: f32 = (MOVE_CYCLES * FREQ_LOOP_MOVES) as f32 / (clock as f32);
	let t_halt: f32 = HALT_CYCLES as f32 / (clock as f32);
	let mut out = String::new();
	for i in 0..256 {
		out.push_str(
			&format!(
				"write {:#04X} gpram-inc-addr;\n",
				255 - if i == 0 {// Subtract from 255 because the further into the halt block, the less time is spent in it
					255u8
				}
				else {
					let total_duration: f32 = 1.0 / (i as f32);
					let halts_raw: f32 = (total_duration - t_loop) / t_halt;
					if halts_raw >= 255.0 {
						255
					}
					else {
						halts_raw as u8
					}
				}
			)
		);
	}
	// Done
	out
}

/// Song is a Vec of (Frequency (Hz), Duration (ms))
fn generate_song_sequence(song: &Vec<(u32, u32)>, clock: u32) -> String {
	let t_outer_loop_ms: f32 = 1000.0 * (MOVE_CYCLES * OUTER_LOOP_MOVES) as f32 / (clock as f32);
	let t_min: f32 = (MOVE_CYCLES * FREQ_LOOP_MOVES) as f32 / (clock as f32);
	let t_max: f32 = (MOVE_CYCLES * FREQ_LOOP_MOVES + HALT_CYCLES * 255) as f32 / (clock as f32);
	println!("T min: {} ms, T max: {} ms, T outer loop: {} ms", t_min * 1000.0, t_max * 1000.0, t_outer_loop_ms);
	let f_max: f32 = 1.0 / t_min;
	let f_min: f32 = 1.0 / t_max;
	println!("F max: {} Hz, F min: {} Hz", f_max, f_min);
	// Check if song is within limtits
	let mut f_song_min = song[0].0;
	let mut f_song_max = song[0].0;
	if song.len() >= 128 {
		panic!("Song can't be more then 127 notes");
	}
	for i in 1..song.len() {
		if song[i].0 >= 256 {
			panic!("Frequency can't be higher then 255 Hz");
		}
		if song[i].0 == 0 {
			continue;
		}
		if f_song_min > song[i].0 {
			f_song_min = song[i].0
		}
		if f_song_max < song[i].0 {
			f_song_max = song[i].0
		}
	}
	println!("F song max: {} Hz, F song min: {} Hz", f_song_max, f_song_min);
	if f_song_min < f_min as u32 || f_song_max > f_max as u32 {
		panic!("Song does not fit within range");
	}
	// Create song sequence
	let mut out = String::new();
	for (freq, dur_ms_raw) in song {
		let dur_ms: u32 = *dur_ms_raw - (t_outer_loop_ms as u32);
		let cycles_raw: u32 = if *freq == 0 {// Special case if frequency is 0, lookup table will map 0 Hz to max delay
			((dur_ms as f32 / 1000.0) / t_max) as u32
		}
		else {
			((dur_ms as f32 / 1000.0) * (*freq as f32)) as u32
		};
		if cycles_raw > 255 {
			panic!("Number of cycles required to delay note ({}) is greater then 255", cycles_raw);
		}
		let cycles = cycles_raw as u8;
		out.push_str(
			&format!(
				"write {:#04X} gpram-inc-addr;write {:#04X} gpram-inc-addr;# Duration={}, Freq={}\n",// Duration (cycles), Frequency
				cycles,
				freq,
				dur_ms as f32 / 1000.0,
				freq
			)
		);
	}
	out.push_str(&format!("write {:#04X} stack-push;\n", song.len() * 2));
	// Done
	out
}

/// Song is a Vec of (Frequency (Hz), Duration (ms))
fn generate_song_sequence_direct_lookup(song: &Vec<(u32, u32)>, clock: u32) -> String {
	let t_outer_loop_ms: f32 = 1000.0 * (MOVE_CYCLES * OUTER_LOOP_MOVES) as f32 / (clock as f32);
	let t_halt: f32 = HALT_CYCLES as f32 / (clock as f32);
	let t_min: f32 = (MOVE_CYCLES * FREQ_LOOP_MOVES_DIRECT_LOOKUP) as f32 / (clock as f32);
	let t_max: f32 = (MOVE_CYCLES * FREQ_LOOP_MOVES_DIRECT_LOOKUP + HALT_CYCLES * 255) as f32 / (clock as f32);
	println!("T min: {} ms, T max: {} ms, T outer loop: {} ms", t_min * 1000.0, t_max * 1000.0, t_outer_loop_ms);
	let f_max: f32 = 1.0 / t_min;
	let f_min: f32 = 1.0 / t_max;
	println!("F max: {} Hz, F min: {} Hz", f_max, f_min);
	// Check if song is within limtits
	let mut f_song_min = song[0].0;
	let mut f_song_max = song[0].0;
	if song.len() >= 128 {
		panic!("Song can't be more then 127 notes");
	}
	for i in 1..song.len() {
		if song[i].0 == 0 {
			continue;
		}
		if f_song_min > song[i].0 {
			f_song_min = song[i].0
		}
		if f_song_max < song[i].0 {
			f_song_max = song[i].0
		}
	}
	println!("F song max: {} Hz, F song min: {} Hz", f_song_max, f_song_min);
	if f_song_min < f_min as u32 || f_song_max > f_max as u32 {
		panic!("Song does not fit within range");
	}
	// Create song sequence
	let mut out = String::new();
	for (freq, dur_ms_raw) in song {
		let dur_ms: u32 = *dur_ms_raw - (t_outer_loop_ms as u32);
		let cycles_raw: u32 = if *freq == 0 {// Special case if frequency is 0, lookup table will map 0 Hz to max delay
			((dur_ms as f32 / 1000.0) / t_max) as u32
		}
		else {
			((dur_ms as f32 / 1000.0) * (*freq as f32)) as u32
		};
		if cycles_raw > 255 {
			panic!("Number of cycles required to delay note ({}) is greater then 255", cycles_raw);
		}
		let cycles = cycles_raw as u8;
		let halts: u8 = {
			let total_duration: f32 = 1.0 / (*freq as f32);
			let halts_raw: f32 = (total_duration - t_min) / t_halt;
			assert!(halts_raw <= 255.0, "Halts required is > 255, this should have been caught by earlier code");
			assert!(halts_raw >= 0.0, "Halts required is < 0, this should have been caught by earlier code");
			halts_raw as u8
		};
		out.push_str(
			&format!(
				"write {:#04X} gpram-inc-addr;write {:#04X} gpram-inc-addr;# Duration={}, Freq={}\n",// Duration (cycles), goto-a value
				cycles,
				255 - halts,
				dur_ms as f32 / 1000.0,
				freq
			)
		);
	}
	out.push_str(&format!("write {:#04X} stack-push;\n", song.len() * 2));
	// Done
	out
}

pub fn print_output(clock: u32, include_lookup_table: bool, direct_lookup: bool) {
	// Imperial march
	//let song_freqs: Vec<f32> = vec![110.0, 87.25, 130.75, 110.0, 87.25, 130.75, 110.0, 164.75, 0.0, 164.75, 0.0, 164.75, 0.0, 174.5, 130.75, 103.75, 87.25, 130.75, 110.0, 220.0, 110.0, 220.0, 207.5, 196.0, 185.0, 174.5, 185.0, 0.0, 113.75, 155.5, 146.75, 138.5, 130.75, 116.5, 130.75, 0.0, 87.25, 103.75, 87.25, 110.0, 130.75, 110.0, 130.75, 164.75, 220.0, 110.0, 110.0, 220.0, 207.5, 196.0, 185.0, 174.5, 185.0, 0.0, 113.75, 155.5, 146.75, 138.5, 130.75, 116.5, 130.75, 0.0, 87.25, 103.75, 87.25, 130.75, 110.0, 87.25, 65.25, 110.0];
	//let song_durations: Vec<u32> = vec![150, 350, 150, 500, 350, 150,1000, 400, 100, 400, 100, 400, 100, 350, 150, 500, 350, 150,1000, 500, 500, 500, 250, 250, 125, 125, 250, 250, 250, 500, 250, 250, 125, 125, 250, 250, 125, 500, 375, 125, 500, 375, 125,1000, 500, 350, 150, 500, 250, 250, 125, 125, 250, 250, 250, 500, 250, 250, 125, 125, 250, 250, 250, 500, 375, 125, 500, 375, 125,1000];
	//let scale: f32 = 0.9;
	// Duel of the Fates
	//let song_freqs: Vec<f32> = vec![195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 329.0, 349.0, 391.0, 440.0, 233.0, 440.0, 391.0, 349.0, 329.0, 0.0, 329.0, 349.0, 391.0, 440.0, 233.0, 440.0, 391.0, 349.0, 329.0, 0.0, 238.0, 329.0, 349.0, 391.0, 220.0, 466.0, 440.0, 391.0, 349.0, 0.0, 238.0, 329.0, 349.0, 391.0, 220.0, 391.0, 349.0, 329.0, 238.0, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0];
	//let song_durations: Vec<u32> = vec![197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 197, 197, 395, 395, 395, 395, 197, 197, 395, 395, 197, 197, 395, 395, 395, 395, 197, 197, 395, 395, 197, 197, 395, 395, 395, 395, 197, 197, 395, 395, 197, 197, 395, 395, 395, 395, 197, 197, 395, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197];
	//let scale: f32 = 0.53*4.0;
	// Final Countdown (intro)
	/*let song_freqs: Vec<f32> = vec![261.0, 466.0, 261.0, 349.0, 349.0, 0.0, 294.0, 261.0, 294.0, 261.0, 466.0, 466.0, 0.0, 294.0, 261.0, 294.0, 349.0, 349.0, 0.0, 466.0, 440.0, 466.0, 440.0, 391.0, 466.0, 440.0, 440.0, 261.0, 466.0, 261.0, 349.0, 349.0, 0.0, 294.0, 261.0, 294.0, 261.0, 466.0, 466.0, 0.0, 294.0, 261.0, 294.0, 349.0, 349.0, 0.0, 466.0, 440.0, 466.0, 440.0, 391.0, 466.0, 440.0, 440.0, 391.0, 440.0, 466.0, 466.0, 440.0, 466.0, 261.0, 466.0, 440.0, 391.0, 349.0, 294.0, 261.0, 261.0, 261.0, 294.0, 294.0, 261.0, 466.0, 261.0, 261.0, 261.0, 261.0];
	let song_durations: Vec<u32> = vec![99, 99, 395, 395, 197, 395, 99, 99, 197, 197, 395, 197, 395, 99, 99, 395, 395, 197, 395, 99, 99, 197, 197, 197, 197, 395, 197, 99, 99, 395, 395, 197, 395, 99, 99, 197, 197, 395, 197, 395, 99, 99, 395, 395, 197, 395, 99, 99, 197, 197, 197, 197, 395, 197, 99, 99, 395, 197, 99, 99, 197, 197, 197, 197, 395, 395, 395, 395, 197, 99, 197, 99, 99, 395, 395, 395, 395];
	let scale: f32 = 0.53;*/
	// Haaland TODO
	let song_freqs: Vec<f32> = vec![195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 329.0, 349.0, 391.0, 440.0, 233.0, 440.0, 391.0, 349.0, 329.0, 0.0, 329.0, 349.0, 391.0, 440.0, 233.0, 440.0, 391.0, 349.0, 329.0, 0.0, 238.0, 329.0, 349.0, 391.0, 220.0, 466.0, 440.0, 391.0, 349.0, 0.0, 238.0, 329.0, 349.0, 391.0, 220.0, 391.0, 349.0, 329.0, 238.0, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 195.5, 220.0, 195.5, 174.5, 164.5, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0, 391.0, 440.0, 391.0, 349.0, 329.0];
	let song_durations: Vec<u32> = vec![197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 197, 197, 395, 395, 395, 395, 197, 197, 395, 395, 197, 197, 395, 395, 395, 395, 197, 197, 395, 395, 197, 197, 395, 395, 395, 395, 197, 197, 395, 395, 197, 197, 395, 395, 395, 395, 197, 197, 395, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197, 197, 197, 99, 99, 197];
	let scale: f32 = 0.53*4.0;
	// Test
	//let song_freqs: Vec<f32> = vec![100.0, 0.0, 78.0, 0.0, 255.0, 0.0, 78.0, 0.0, 255.0, 0.0, 78.0];
	//let song_durations: Vec<u32> = vec![500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500];
	//let scale: f32 = 1.0;
	assert!(song_freqs.len() == song_durations.len());
	// Compile song into currect format
	let mut song = Vec::<(u32, u32)>::new();
	for i in 0..song_freqs.len() {
		song.push(((song_freqs[i] * scale) as u32, song_durations[i]));
	}
	if include_lookup_table && !direct_lookup {
		println!("Lookup table:\n```\n{}```", generate_lookup_table(clock));
	}
	println!(
		"Song:\n```\n{}```",
		if direct_lookup {
			generate_song_sequence_direct_lookup(&song, clock)
		}
		else {
			generate_song_sequence(&song, clock)
		}
	);
}