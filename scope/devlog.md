# Devlog

### Three Features

2021-06-13

The way I imagine it working is recording the run of a program.  Like how you'd profile the run of a program, but instead it would record the state of variables and data structures.

Then you could play back the run, visualizing the data structures over time.  And playing it back like a movie with a scrubber that allows you to jump around would look cool.  But I'm not sure how useful that would be.  I think if I wanted to use it for actual development, I'd want a few features beyond that.

Note that when I say "state", I mean _runtime_ state.

1. **Visualize state over time.**  Following Bret Victor's lead, one example would be graphing the value of a numeric value over time.  It basically converts the time dimension into a spatial one.  You can do it because you have all the data over the course of a program run at once.  Not sure how this would work for data structures, but I'd start simple.
2. **Query the state, across all time.**  When debugging normally, you'd set a breakpoint in a debugger with a condition to break like when `x % 3 == 0`.  Since this new tool has a recording of all the state over time, you could ask it, _show me all the times when `x % 3 == 0`_, not just the first time.  And like Google searching, once you see the results, you often come up with a new question.  And you could ask it a new question with a different condition without rerunning the program.  Imagine this displaying results instantly as you finish typing the condition.
3. **Causation analysis.**  This is the killer feature I want.  It's related to data provenance, but with a different goal.  Once I've worked on a project for a while, a _major_ value-add from my familiarity with the project often boils down to answering the question: _if we want to change the output of a program from X to Y, what is the code we need to change to cause this change and only this change?_  I imagine that this debugger/visualizer/scope tool could help answer that question.  It's the question of, given an effect, what is the cause of it?  The cause is both data and code.  This is challenging, in part because there isn't just one cause, but many causes that branch backward in time.  But I think we could go a long way to a tool that could help with this.  It's a feature that Eve -- the datalog implementation in the browser -- claimed to do.  But of course, that was a whole other programming paradigm.  I think that just surfacing and showing you the dataflow from a typical imperative program would be a big part of answering questions like this.  The question is, can this be done in a way that is comprehensible and navigable?
