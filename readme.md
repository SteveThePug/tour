# Goals

The purpose of this command will be to walk through code tutorials

It will be essentially the same as git, however without branches.

Through a series of commits, you can walk through code changes and see what has changed (like git diff)

It will work as such

````
```sh
# The format of this is:

command args
# ->
# output
# output line 2

# CREATING A TOUR
tour init -m "After running *cargo init* we get our template"
tour commit src/lib.rs -m "In lib.rs we add some functions"
tour commit src/main.rs -m "We import our newly made function from lib.rs"
tour end -m "Now your tour is complete and you can use rust modules!"

tour start
# ->
# New files:
# ./cargo.lock
# ./cargo.toml
# src/main.rs
#
# Explanation
# After running *cargo init* we get our template

tour next
# ->
# New files:
# src/lib.rs
# Explanation
# In lib.rs we add some functions

tour prev
# ->
# New files:
# ./cargo.lock
# ./cargo.toml
# src/main.rs
#
# Explanation
# After running *cargo init* we get our template

tour next 2
# ->
# No new files.

# Changes:
# src/main.rs

# Explanation
# We import our newly made function from lib.rs

tour next
# ->
# Tutorial Finished
# Explanation
# Now your tour is complete and you can use rust modules!

# EXTRAS:
tour author -> Add information about the author if there are questions
```
````
