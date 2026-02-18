# Goals

The purpose of this command will be to walk through code tutorials

It will be essentially the same as git, however without branches.

Through a series of commits, you can walk through code changes and see what has changed (like git diff)

It will work as such

````
```sh
# When creating a tour
tour init -m "After running *npm init* we get our template"

tour commit file1.sh file2.sh -m "In file1 we add ..., in file2 we add ..."

tour end -m "Now your tour is complete!"

# EXTRAS:
tour author -> Add information about the author if there are questions


tour start
New files:
file1.sh
file2.sh

```
````
