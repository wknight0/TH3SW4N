**TH3SW4N**

**_This is classed as malware so please don't download if you don't know what you're doing. I'm sharing this for educational and reverse engineering purposes._** 

**_Even though there are safeguards in place, you will be held liable for any damages caused!!!_** 

![image](https://github.com/wknight0/TH3SW4N/assets/109128542/b927f23a-1d02-4825-8b32-f4798e8b288b)

This program was made utilizing specific Rust dependencies which can be located in the Cargo.toml file, along with use of GTK4 for creating the GUI.

In order to set up the program using the master branch, follow these steps for Windows:
1. Clone repository
2. Install rustup (https://www.rust-lang.org/tools/install)
3. Install gtk4 (https://www.gtk.org/docs/installations/windows/)
4. (I highly recommend you run on a virtual machine but if so, I would avoid using VirtualBox as the MESA rendering is buggy when compiling gtk)

The program does the following in order:
1. Upon startup, it will prompt the user to confirm that they will be executing malware which will have detrimental effects on their computer.
2. The main application will then wait until it has searched directories and stored file paths to the users files, encrypting them immediately after
3. The application will create a swan.txt file on the desktop which will display the code needed to use to keep the malware at standby
4. Then the main GUI will open, initiailizing a failsafe just for show before beginning a countdown after a random time has passed
5. Only after the 60 mark on the timer, will the user be able to interact with the code buttons.
6. Entering the correct code before clicking the execute button will decrypt 5 user files before resetting the countdown to its original state (start of loop)
7. Entering the wrong code will simply clear the textview and allow the user to try input another code
8. When the countdown reaches zero, it will begin the system_failure sequence which will delete between 1 to 20 random system files, before resetting the program and countdown back to its original state (start of loop)

Feel free to add on to the program, there are many further ideas that could be implemented, such as propagation features, compiling a simplified build, and so on...

This malware was heavily inspired and replicated (same assets used) based on TH3SW4N virus created and used by the team that developed the Welcome to the Game 2+ mod (https://wttg2plus.ampersoft.cz/)
