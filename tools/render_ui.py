#!/usr/bin/env python3
"""Render NyashkAir's faceplate (see hwrender.py). Run from the repo root."""
import os, sys
sys.path.insert(0, os.path.dirname(__file__))
from hwrender import *

ACC = (60, 160, 255)
p = Panel(640, 430, (0.80, 0.81, 0.82), ACC, (30, 34, 40, 240), (80, 88, 98, 230), light_panel=True, wear=0.8)
header(p, 'AIR', 'PRESENCE & AIR EXCITER', x0=36)
p.button('power', 570, 32, 64, 22, 'POWER')
p.screen('curve', 36, 74, 430, 140)
p.text('AIR CURVE  ·  SPECTRUM IN / OUT', 251, 226, 7.2, None, True, sp=1)
lad = [(0.0, ACC), (0.75, (255, 190, 50)), (0.9, (255, 70, 60))]
p.ladder('in', 486, 74, 16, 140, 22, lad); p.ladder('out', 514, 74, 16, 140, 22, lad)
p.text('IN', 494, 226, 7.2, None, True); p.text('OUT', 522, 226, 7.2, None, True)
p.text('ADDED', 578, 84, 7, None, True, sp=1)
p.readout('added', 578, 104, size=12, col=(20, 24, 30, 255))
p.text('vs existing', 578, 124, 6.5); p.text('highs', 578, 135, 6.5)
labels = [str(v) for v in range(0, 101, 10)]
for id, cx, lbl, sub in [('mid', 130, 'MID AIR', '2 - 8 kHz'), ('high', 380, 'HIGH AIR', '8 kHz +')]:
    p.knob(id, cx, 318, 96, 'alu', label=lbl, label_dy=78, lsz=9.5, lcol=p.ink,
           scale=dict(r=62, n=41, major=4, labels=labels, lsz=7.5), led=dict(r=53, n=31))
    p.text(sub, cx, 410, 7)
p.button('link', 255, 318, 54, 22, 'LINK')
p.knob('trim', 555, 318, 56, 'alu', label='TRIM', label_dy=48, lsz=8, scale=dict(r=34, n=11, tick=4, labels=['-12', '', '', '', '', '0', '', '', '', '', '+12'], lsz=6.5), led=dict(r=28, n=17), bip=True)
p.export(os.path.join(os.path.dirname(__file__), '..'), 'plugins/nyashkair')
