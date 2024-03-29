[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/iYoQzOhX)
# Rustybox
Rustybox is a copy of BusyBox written in Rust.
In the following paragraphs I will describe my approach for each implemented function.

For parsing the input I decided to match the first argument with the commands and send the other CLI arguments as parameters.

### pwd
Fetched the current dir with `env::current_dir()` and displayed it. The homework didn't mention an Error code in case of a pwd fail, so I used -2.

### echo
Checked the first parameters for flags and printed the rest. I don't think echo can fail.

### cat
Opens each given file and prints the contents. (Using `fs::File::open()` and `read_to_string()`.)

### mkdir
Creates every given directory using `fs::create_dir`.

### mv
Because mv either moves or renames files, I had to break it into some cases:
1 - moving a file to a directory
2 - renaming a file
3 - renaming a directory.

### ln
Symlink and hard_link are the functions I used to create the links, according to the flags.

### rmdir
`remove_dir` for each given directory name. If the given argument isn't a directory, it fails and an error code is returned.

### rm
Used `remove_file` and my `rmdir` for removing files, also `remove_dir_all` for removing recursively.

### ls
I used `contains()` for looking up the flags and `filter()` for skipping them while iterating. My approach involved using `fs::read_dir` and printing the entries. I had to sort the entries so that the `ls -l` cases didn't differ from the ref.
I decided to make an additional function for the recursive ls because the format differs from the normal one.

`ls -l` was by far the most complex assignment of this homework. I needed many helper functions for computing the information displayed by it. Firstly, I got the permissions using `.permissions().mode()` and matched them with the display type. Then, I got the owner and group by parsing `/etc/passwd` and `/etc/group`. The size of the file is stored in metadata, so it was easy to fetch.
In the end I made a function for formatting the last modify time using `chrono`. All this information was formatted in another function which was called for every file.

### cp
I used `fs::copy` for copying the files. In the case of copying into a directory, I had to format the name of the file so that the new file was inside the directory.

For `cp -r` I called my function recursively, each time creating a `Vec<String>` to be passed as argument, containing the source and destination of the copy, along with the recursive flag.

### touch
I initially looked for functions that would modify the file's times but they were part of the nightly build, so I decided to read from file to modify the access time and write a null byte for updating the modify time. I am looking forward to improving this function.

### chmod
Similarly to `ls -l` I parsed the permissions and used `set_permissions()` to update the files. I also had to converse the permissions to base 8 for easier understanding.

### grep
I used the regex crate for checking weather each line from the given file was a match. Also implemented the `-i`.(the homework mentions `-i` to be inverting the result, corresponding to `-v` for the Linux grep.)

### Notes
I learned a lot about Rust by doing this homework. I tried refactoring some bits of code after learning about new functions but the code might still seem a bit inconsistent.
https://doc.rust-lang.org was extremely useful for researching modules and reading about the methods.
Chat-GPT was also very useful for better understanding the syntax and quickly finding methods, although not always coming up with good answers.

I am looking forward to feedback and ways I could improve my Rust knowledge.
## Verify

Run the following commands to test your homework:

You will have to install NodeJS (it is installed in the codespace)

```bash
# Clone tests repository
git submodule update --init 

# Update tests repository to the lastest version
cd tests
git pull 
cd ..

# Install loadash
npm install lodash
```

Install rustybox

```bash
cargo install --path .
```

If the `rustybox` command can't be found, be sure to add the default cargo installation folder into the PATH environment variable

```bash
export PATH=/home/<your username here>/.cargo/bin:$PATH
```

Run tests

```bash
cd tests
# Run all tests 
./run_all.sh

# Run single test
./run_all.sh pwd/pwd.sh
```
