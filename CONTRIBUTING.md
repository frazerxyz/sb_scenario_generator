## Contributing Airport Data

### Getting configured
1. Install [Rust](https://rust-lang.org/tools/install/)
2. Create a fork of the repo
3. Clone a local copy of your fork using `git clone [your_fork_url]`
4. Create a branch for your changes

### Editing data files
Some of the json files are quite large, using a text editor such as VS Code or Notepad++ that allow you to collapse unused sections might make your life easier.

### Testing
- Verify that your json file is formatted correctly using `cargo test` in terminal from the project root
- Test run the program with your changes using `cargo run`

### Creating a PR
1. Commit your changes and push your branch
2. On GitHub, click "create PR"
3. Check the tests complete successfully and fix the errors if they don't
4. Make sure the PR is marked as "Ready for review"