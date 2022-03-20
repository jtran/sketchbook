Astronomers have the telescope.

Biologists have the microscope.

What do software engineers have?

I propose a scope for inspecting the runtime state of software systems.

# Motivation

At Carnegie Mellon where I studied computer science, professors taught me how to think about algorithms using diagrams specifically designed to convey relevant information.  Then, when I went to actually implement the algorithms, I had to simulate the computer in my mind and use textual print statements to see the results.

The human mind has a finite capacity.  Instead of people having to simulate what a computer would do, why don't computers just do it and show you what they did?

Up until now, there have been many attempts at visual programming environments.

In my opinion, prior attempts have failed because they visualize the wrong thing.  They visualize the code, not the runtime state.

In software development, a significant amount of time and energy is spent reconstructing the runtime state of complex programs in the mind of developers.

When developers attempt to make a change to software, they need a map of the data flow in their mind so that they can work backwards from the desired change to the code that causes that change.  Then they have to simulate what the computer would do to determine if their changes would produce the desired result.  Since software is complex, there are inevitably differences which lead to bugs.

Runtime state of a program can be extremely complex, and it changes over time as the program runs.  Text or even static diagrams are insufficient.  Like most new technologies imitating prior ones, much of computing history has been replicating what used to be done on paper.

I believe that there's still a lot of untapped potential to break free of replicating static print, and doing so will be revolutionary.  These ideas aren't new.  I'm just restating what visionaries have been saying for years.

So to make this a reality, I'm building an app called Visionnn that uses principles from graphic design and game development to convey program state in a way that matches how universities teach computer science.

Since all the best developers I know are very opinionated about the programming languages and development environments they use, Visionnn is designed to work with any language by using an open protocol that works on any platform.

In the same way that JSON is the lowest common denominator across languages for representing a data structure, at a point in time, Visionnn's protocol uses simple operations that are common across languages for representing state change, throughout the run of a program.

Instead of trying to create an entirely new programming language, runtime, and IDE, Visionnn's goal isn't to reinvent the wheel or try to convince people to switch from the tools they're already using.  Visionnn is designed to do one thing well -- convey runtime state -- that can augment whatever you're already doing.

Visionnn enables you to:

- Increase your productivity by freeing your mind from simulating the computer
- Communicate how complex processes work more effectively than co-workers by using interactive animations
- See how data flows through a system, even when there's no documentation
- Reveal bugs and inefficiencies in ways that log files can't
- Find the code that causes the effects you want in an unfamiliar codebase
- Save time and avoid tedious, repetitive testing by changing breakpoint expressions after the program has already run

I call it Visionnn because I truly believe that once you become accustomed to a workflow with Visionnn, making software without it will feel like flying blind.

There's currently a working prototype of Visionnn, and I use it to dogfood its own development.

Sign up for my mailing list now to be among the first to be notified of its release.  People who sign up first will be prioritized for private beta releases.

# History

This directory is an umbrella for multiple projects with several incarnations
and re-writes over the years, all with the same theme.

[machina](https://github.com/jtran/machina) is one of the earlier incarnations,
but not the first.

Over the years, I've written projects in C++, OCaml, JavaScript, Java, Rust,
and most recently, CoffeeScript.
