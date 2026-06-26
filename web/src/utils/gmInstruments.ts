import type { SoundfontValue } from './partSource'

export type InstrumentCategory =
  | 'piano'
  | 'chromatic-perc'
  | 'organ'
  | 'guitar'
  | 'bass'
  | 'strings'
  | 'ensemble'
  | 'brass'
  | 'reed'
  | 'pipe'
  | 'synth-lead'
  | 'synth-pad'
  | 'synth-fx'
  | 'ethnic'
  | 'percussive'
  | 'sound-fx'

export type InstrumentSource = 'acoustic' | 'synth'
export type InstrumentRole = 'melody' | 'bass' | 'pad' | 'rhythm'
export type InstrumentArticulation =
  | 'plucked'
  | 'bowed'
  | 'blown'
  | 'struck'
  | 'electronic'
  | 'vocal'

export interface InstrumentEntry {
  value: SoundfontValue
  category: InstrumentCategory
  source: InstrumentSource
  role: InstrumentRole
  articulation: InstrumentArticulation
}

// biome-ignore format: large data table — one entry per line for readability
export const GM_INSTRUMENTS: InstrumentEntry[] = [
  // Piano (0–7)
  { value: '0: Acoustic Grand Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '1: Bright Acoustic Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '2: Electric Grand Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '3: Honky-tonk Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '4: Electric Piano 1', category: 'piano', source: 'synth', role: 'melody', articulation: 'struck' },
  { value: '5: Electric Piano 2', category: 'piano', source: 'synth', role: 'melody', articulation: 'struck' },
  { value: '6: Harpsichord', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '7: Clavi', category: 'piano', source: 'synth', role: 'melody', articulation: 'struck' },
  // Chromatic Perc (8–15)
  { value: '8: Celesta', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '9: Glockenspiel', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '10: Music Box', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '11: Vibraphone', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '12: Marimba', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '13: Xylophone', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '14: Tubular Bells', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '15: Dulcimer', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  // Organ (16–23)
  { value: '16: Drawbar Organ', category: 'organ', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '17: Percussive Organ', category: 'organ', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '18: Rock Organ', category: 'organ', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '19: Church Organ', category: 'organ', source: 'acoustic', role: 'pad', articulation: 'blown' },
  { value: '20: Reed Organ', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '21: Accordion', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '22: Harmonica', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '23: Tango Accordion', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Guitar (24–31)
  { value: '24: Acoustic Guitar (nylon)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '25: Acoustic Guitar (steel)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '26: Electric Guitar (jazz)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '27: Electric Guitar (clean)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '28: Electric Guitar (muted)', category: 'guitar', source: 'acoustic', role: 'rhythm', articulation: 'plucked' },
  { value: '29: Overdriven Guitar', category: 'guitar', source: 'synth', role: 'melody', articulation: 'plucked' },
  { value: '30: Distortion Guitar', category: 'guitar', source: 'synth', role: 'melody', articulation: 'plucked' },
  { value: '31: Guitar Harmonics', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  // Bass (32–39)
  { value: '32: Acoustic Bass', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { value: '33: Electric Bass (finger)', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { value: '34: Electric Bass (pick)', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { value: '35: Fretless Bass', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { value: '36: Slap Bass 1', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { value: '37: Slap Bass 2', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { value: '38: Synth Bass 1', category: 'bass', source: 'synth', role: 'bass', articulation: 'electronic' },
  { value: '39: Synth Bass 2', category: 'bass', source: 'synth', role: 'bass', articulation: 'electronic' },
  // Strings (40–47)
  { value: '40: Violin', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { value: '41: Viola', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { value: '42: Cello', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { value: '43: Contrabass', category: 'strings', source: 'acoustic', role: 'bass', articulation: 'bowed' },
  { value: '44: Tremolo Strings', category: 'strings', source: 'acoustic', role: 'pad', articulation: 'bowed' },
  { value: '45: Pizzicato Strings', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '46: Orchestral Harp', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '47: Timpani', category: 'strings', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  // Ensemble (48–55)
  { value: '48: String Ensemble 1', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'bowed' },
  { value: '49: String Ensemble 2', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'bowed' },
  { value: '50: Synth Strings 1', category: 'ensemble', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '51: Synth Strings 2', category: 'ensemble', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '52: Choir Aahs', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'vocal' },
  { value: '53: Voice Oohs', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'vocal' },
  { value: '54: Synth Voice', category: 'ensemble', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '55: Orchestra Hit', category: 'ensemble', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  // Brass (56–63)
  { value: '56: Trumpet', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '57: Trombone', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '58: Tuba', category: 'brass', source: 'acoustic', role: 'bass', articulation: 'blown' },
  { value: '59: Muted Trumpet', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '60: French Horn', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '61: Brass Section', category: 'brass', source: 'acoustic', role: 'pad', articulation: 'blown' },
  { value: '62: Synth Brass 1', category: 'brass', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '63: Synth Brass 2', category: 'brass', source: 'synth', role: 'melody', articulation: 'electronic' },
  // Reed (64–71)
  { value: '64: Soprano Sax', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '65: Alto Sax', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '66: Tenor Sax', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '67: Baritone Sax', category: 'reed', source: 'acoustic', role: 'bass', articulation: 'blown' },
  { value: '68: Oboe', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '69: English Horn', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '70: Bassoon', category: 'reed', source: 'acoustic', role: 'bass', articulation: 'blown' },
  { value: '71: Clarinet', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Pipe (72–79)
  { value: '72: Piccolo', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '73: Flute', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '74: Recorder', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '75: Pan Flute', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '76: Blown Bottle', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '77: Shakuhachi', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '78: Whistle', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '79: Ocarina', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Synth Lead (80–87)
  { value: '80: Lead 1 (square)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '81: Lead 2 (sawtooth)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '82: Lead 3 (calliope)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '83: Lead 4 (chiff)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '84: Lead 5 (charang)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '85: Lead 6 (voice)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '86: Lead 7 (fifths)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { value: '87: Lead 8 (bass + lead)', category: 'synth-lead', source: 'synth', role: 'bass', articulation: 'electronic' },
  // Synth Pad (88–95)
  { value: '88: Pad 1 (new age)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '89: Pad 2 (warm)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '90: Pad 3 (polysynth)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '91: Pad 4 (choir)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '92: Pad 5 (bowed)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '93: Pad 6 (metallic)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '94: Pad 7 (halo)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '95: Pad 8 (sweep)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  // Synth FX (96–103)
  { value: '96: FX 1 (rain)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '97: FX 2 (soundtrack)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '98: FX 3 (crystal)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '99: FX 4 (atmosphere)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '100: FX 5 (brightness)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '101: FX 6 (goblins)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '102: FX 7 (echoes)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { value: '103: FX 8 (sci-fi)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  // Ethnic (104–111)
  { value: '104: Sitar', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '105: Banjo', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '106: Shamisen', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '107: Koto', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '108: Kalimba', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { value: '109: Bag Pipe', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { value: '110: Fiddle', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { value: '111: Shanai', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Percussive (112–119)
  { value: '112: Tinkle Bell', category: 'percussive', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '113: Agogo', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { value: '114: Steel Drums', category: 'percussive', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { value: '115: Woodblock', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { value: '116: Taiko Drum', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { value: '117: Melodic Tom', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { value: '118: Synth Drum', category: 'percussive', source: 'synth', role: 'rhythm', articulation: 'electronic' },
  { value: '119: Reverse Cymbal', category: 'percussive', source: 'synth', role: 'rhythm', articulation: 'electronic' },
  // Sound FX (120–127)
  { value: '120: Guitar Fret Noise', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'plucked' },
  { value: '121: Breath Noise', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'blown' },
  { value: '122: Seashore', category: 'sound-fx', source: 'acoustic', role: 'pad', articulation: 'electronic' },
  { value: '123: Bird Tweet', category: 'sound-fx', source: 'acoustic', role: 'melody', articulation: 'vocal' },
  { value: '124: Telephone Ring', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'electronic' },
  { value: '125: Helicopter', category: 'sound-fx', source: 'acoustic', role: 'pad', articulation: 'electronic' },
  { value: '126: Applause', category: 'sound-fx', source: 'acoustic', role: 'pad', articulation: 'vocal' },
  { value: '127: Gunshot', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'electronic' },
]
