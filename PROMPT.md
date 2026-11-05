BRANCH=delete
REMOTE=fork
ISSUES=30-34

1. read ./planning/delete-design.md
2. inspect every issue in <ISSUES>, with comments: use `gh issue view <N> -R natefduncan/sheetsui --comments` (plain `gh issue view` shows the body only and hides the comments)
3. check commit history (`git log --oneline main..<BRANCH>`) and the current tree for work already implemented
4. select the first issue from <ISSUES> that is open, **preserving the listed order**, and whose "Blocked by" issues are all closed
5. implement it test-first, following the design doc's decisions (R1-R9) and the existing code style
6. update the docs the issue names
7. commit with a message that closes the issue when merged (`Closes #<N>`)
8. push to <REMOTE>/<BRANCH>

iter is finished when you've pushed to <BRANCH>
loop is finished when all issues in <ISSUES> are closed

IMPORTANT:

- do not commit any ralph files
- one issue per ralph iteration
- make sure that you generate the next task for the next iter with `ralph` CLI before you finish the iter
