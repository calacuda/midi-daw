from midi_daw_types import MidiChannel, NoteLen

from .main import *

Ch1 = MidiChannel.Ch1
Ch2 = MidiChannel.Ch2
Ch3 = MidiChannel.Ch3
Ch4 = MidiChannel.Ch4
Ch5 = MidiChannel.Ch5
Ch6 = MidiChannel.Ch6
Ch7 = MidiChannel.Ch7
Ch8 = MidiChannel.Ch8
Ch9 = MidiChannel.Ch9
Ch10 = MidiChannel.Ch10
Ch11 = MidiChannel.Ch11
Ch12 = MidiChannel.Ch12
Ch13 = MidiChannel.Ch13
Ch14 = MidiChannel.Ch14
Ch15 = MidiChannel.Ch15
Ch16 = MidiChannel.Ch16

# Wn = NoteLen.Wn
# Hn = NoteLen.Hn
# Qn = NoteLen.Qn
# En = NoteLen.En
# Sn = NoteLen.Sn
# Tn = NoteLen.Tn
# S4n = NoteLen.S4n


def wn(n=1):
    return NoteLen.Wn(n)


def hn(n=1):
    return NoteLen.Hn(n)


def qn(n=1):
    return NoteLen.Qn(n)


def en(n=1):
    return NoteLen.en(n)


def sn(n=1):
    return NoteLen.sn(n)


def tn(n=1):
    return NoteLen.Tn(n)


def s4n(n=1):
    return NoteLen.S4n(n)
