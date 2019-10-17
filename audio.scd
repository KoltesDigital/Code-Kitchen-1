(
Routine.run {
	s.bootSync;

	//by nicolaariutti
	//http://sccode.org/1-5as
	SynthDef(\violin, { | freq = 400, amp = 0.8, duration = 0.5 |
		var env = EnvGen.kr(Env(times: [0.1, duration]), doneAction: 2);
		var sig = VarSaw.ar(freq, width: LFNoise2.kr(1).range(0.2, 0.8)*SinOsc.kr(5, Rand(0.0, 1.0)).range(0.7,0.8))*0.25;
		sig = sig * env * amp;
		Out.ar(0, sig!2);
	}).add;

	s.sync;

	o = OSCFunc({ | msg, time, addr, recvPort |
		var count = 10;
		var delay = 0.1;
		var baseNote = 4.rand2;
		var octave = 3;
		var stepsPerOctave = 12;

		"Received at time %.".format(time).postln;

		count.do { | i |
			s.makeBundle(i * delay, {
				var degree = if (i % 2 == 0) { baseNote } { baseNote + 1 };
				var note = degree.degreeToKey(Scale.major, stepsPerOctave);
				var midinote = (note / stepsPerOctave + octave) * 12.0;
				var freq = midinote.midicps * 4.0;
				x = Synth.new(\violin, [\freq, freq, \duration, delay]);
			});
		};

		s.makeBundle(count * delay, {
			var degree = baseNote;
			var note = degree.degreeToKey(Scale.major, stepsPerOctave);
			var midinote = (note / stepsPerOctave + octave) * 12.0;
			var freq = midinote.midicps * 4.0;
			x = Synth.new(\violin, [\freq, freq, \duration, 2.0]);
		});

	}, '/');

	"Ready!".postln;
};
)

NetAddr.langPort; 