# Aurae Examples

Run the examples like you would run a script.

You may run directly from the same directory as long as the script is marked at executable

```bash 
chmod +x my_script.ts
```

and the script contains a valid [shebang](https://en.wikipedia.org/wiki/Shebang_(Unix)) line at the top.

```bash
#!/usr/bin/env auraescript
```
You may run the script as follows:

```bash 
./my_script.ts
```

Additionally you can leverage the `auraescript` binary directly to execute your script which will execute a script without being executable and without a shebang. 

```bash 
auraescript myscript.ts
```