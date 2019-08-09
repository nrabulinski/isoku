![](logo.png)

易速 is a Bancho emulator written for Uncho, an osu! private server developed completely from scratch. In the future we might use a proper HTTP library instead of handling requests and multithreading on our own, but for now it's supposed to be very simple (易しい) and work (and do it relatively fast - 速い).

*Benchmarks comparing competing servers will go here, I guess*

*Also info on how to set this whole thing up etc will be present here in the future, when I'm ready to go opensource*

**very important note** - Enqueueing any data for tokens happens ***only*** inside of events so one can clearly see what events append what packets and easily make changes if necessary.