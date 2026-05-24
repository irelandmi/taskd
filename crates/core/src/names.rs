use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

const ADJECTIVES: &[&str] = &[
	"able", "bold", "calm", "cool", "dark", "deft", "dry", "epic",
	"fair", "fast", "firm", "fond", "free", "full", "glad", "gold",
	"good", "gray", "grim", "hale", "keen", "kind", "late", "lean",
	"live", "long", "loud", "mild", "near", "neat", "next", "nice",
	"pale", "pink", "pure", "rare", "raw", "real", "red", "rich",
	"ripe", "safe", "shy", "slim", "slow", "soft", "sore", "sure",
	"tall", "tame", "thin", "tiny", "top", "true", "vast", "warm",
	"weak", "wide", "wild", "wise", "aged", "apt", "bare", "big",
	"blue", "busy", "cold", "cozy", "crisp", "deep", "dim", "dual",
	"dull", "easy", "even", "fine", "flat", "fresh", "green", "half",
	"hard", "high", "hot", "icy", "idle", "iron", "jade", "just",
	"key", "last", "lax", "lit", "low", "lush", "mad", "main",
	"mute", "new", "odd", "old", "open", "oval", "own", "plain",
	"plum", "posh", "prime", "proud", "quick", "quiet", "rapid", "ready",
	"rigid", "rough", "round", "royal", "rust", "sharp", "sheer", "short",
	"silky", "snug", "solid", "spare", "stark", "steep", "stern", "stiff",
	"stoic", "stout", "swift", "taut", "terse", "thick", "tight", "trim",
	"twin", "vivid", "wary", "wiry", "young", "zany", "zen", "zero",
];

const NOUNS: &[&str] = &[
	"ant", "ape", "bat", "bear", "bee", "bird", "boar", "buck",
	"bull", "calf", "cat", "clam", "claw", "cod", "colt", "crab",
	"crow", "cub", "dart", "deer", "dog", "dove", "duck", "eagle",
	"eel", "elk", "elm", "emu", "fawn", "fig", "fin", "fish",
	"fly", "fog", "fox", "frog", "gem", "goat", "gull", "hare",
	"hawk", "hen", "hog", "horse", "ibis", "jack", "jay", "kite",
	"koi", "lark", "lion", "lynx", "mare", "mink", "mole", "moth",
	"mule", "newt", "node", "oak", "orb", "orca", "osprey", "otter",
	"owl", "ox", "pear", "pike", "pine", "plum", "pony", "pug",
	"quail", "ram", "ray", "reef", "robin", "rook", "rose", "sage",
	"seal", "seed", "slug", "snail", "snake", "sole", "song", "sparrow",
	"squid", "stag", "star", "swan", "teal", "tern", "toad", "trout",
	"tuna", "vale", "vine", "vole", "wasp", "wren", "yak", "yew",
	"bass", "beam", "bolt", "bone", "cape", "cave", "clay", "cliff",
	"coal", "cone", "core", "cove", "crew", "dawn", "dew", "disk",
	"dome", "drum", "dune", "dust", "edge", "fern", "flare", "flint",
	"flux", "foam", "ford", "forge", "frost", "gate", "glen", "glow",
	"grain", "grove", "gust", "helm", "hive", "hook", "hull", "iris",
	"isle", "jade", "jar", "kelp", "knot", "lake", "lamp", "leaf",
	"ledge", "lens", "lime", "loft", "mast", "mesa", "mill", "mint",
	"mist", "moss", "nest", "opal", "palm", "path", "peak", "pond",
	"port", "rain", "reed", "ridge", "rift", "ring", "root", "rust",
	"sand", "shard", "shell", "shore", "silk", "slab", "slate", "slope",
	"snow", "spar", "spire", "spray", "spur", "stem", "stone", "storm",
	"surf", "thorn", "tide", "trail", "tree", "vale", "wave", "well",
	"wind", "wing", "wood", "wool", "yard", "zinc",
];

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn xorshift(mut x: u64) -> u64 {
	x ^= x << 13;
	x ^= x >> 7;
	x ^= x << 17;
	x
}

pub fn generate_name() -> String {
	let nanos = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_nanos() as u64;
	let count = COUNTER.fetch_add(1, Ordering::Relaxed);
	let mut x = xorshift(nanos ^ count.wrapping_mul(0x9e3779b97f4a7c15));
	let adj = ADJECTIVES[(x as usize) % ADJECTIVES.len()];
	x = xorshift(x);
	let noun = NOUNS[(x as usize) % NOUNS.len()];
	x = xorshift(x);
	let suffix = x & 0xffff;
	format!("{adj}-{noun}-{suffix:04x}")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_name_format() {
		let name = generate_name();
		let parts: Vec<&str> = name.split('-').collect();
		assert_eq!(parts.len(), 3);
		assert!(ADJECTIVES.contains(&parts[0]));
		assert!(NOUNS.contains(&parts[1]));
		assert_eq!(parts[2].len(), 4);
	}

	#[test]
	fn test_names_unique() {
		let names: Vec<String> = (0..100).map(|_| generate_name()).collect();
		let unique: std::collections::HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
		assert_eq!(names.len(), unique.len());
	}
}
