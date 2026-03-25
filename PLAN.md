# Plan
This document will outline the plan for developing a fitness note taking app.

## Features
### Core
- Add, edit and view sets, and if sets are personal records (current or at the time).
- Different views
  - Exercise view (see/edit the sets performed on an exercise on a particular day)
    - Secondary views: 
      - History. Show previous days/sets of this exercise
      - Graph. Show interactive graph of date vs metrics (estimated 1rm, PRs, volume, etc)
  - Day view (see/edit the exercises performed on a particular day. Jump to exercise view)
  - Calendar view (see what days exercise was performed/jump to day view)
- Import data tool
  - Should at least work for fitnotes export
- Export data tool
  - To csv
- Body tracker
  - weight, body fat (manual entry or estimate using navy body fat if waist and neck measurements are available), circumference measurements
  - graph view for body tracker metrics
  
### Interesting ideas
- I think modeling the relationship between different metrics could be interesting, weight vs estimated 1rm for instance. 
- Not sure if this is a good concept for strength training, but in cardio world a cumulative load measurement is often useful. Essentially a convolution of daily load with an exponential kernel. Though the daily load is maybe difficult to define for strength, and may be the wrong concept.
