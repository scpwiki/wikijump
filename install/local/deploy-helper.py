import os
import argparse

try:
    import questionary
except ImportError:
    print("questionary mouldal not found. if you are on linux or using wsl2, install it by runing your package managers install command followed by python-questionary. if your on macOS, use homebrew, or if you dont have homebrew, look up how to install questionary")
    sys.exit(1)
def chdir_wikijump():
    # Determine where this script is
    full_path = __file__
    if not os.path.abspath(full_path):
        full_path = os.path.join(os.getcwd(), full_path)

    # The directory it's located in should be install/local/
    # So we should change to that
    install_local_dir = os.path.dirname(full_path)
    os.chdir(install_local_dir)

def is_sudo():
    sudo_choice = questionary.select(
        "Execution Privileges:",
        choices=[
            "Standard User Context (Recommended if possable)",
            "Sudo Context (Run Docker commands via sudo, probly fine, but can cause issues)",
        ],
    ).ask()
    use_sudo = "Sudo Context" in sudo_choice
    return use_sudo
def is_dev():
    dev_choice =  questionary.select(
        "build purpose:",
        choices=[
            "dev envrioment",
            "production envroment",
        ],
    ).ask()
    use_dev = not "production" in dev_choice
    return use_dev
def should_chdir():
    chdir_yn =  questionary.select(
        "should chdir to docker-compose directory?",
        choices=[
            "yes(recomended highly)",
            "no (only use if in the directory of the compose file)",
        ],
    ).ask()
    if "yes" in chdir_yn:
        chdir_wikijump()
def get_docker_action():
    # the other functions only have 2 or 3 choices. however, becouse this one has a bunch of choices, it uses assigned values 
    action = questionary.select(
        "Select docker operation to perform:",
        choices=[
            {
                "name": "up: makes missing images, creates the netowrk, and starts the contaners",
                "value": "up",
            },
            {
                "name": "up --build: runs build before runing up, basicly build but it also starts the contaners",
                "value": "up --build",
            },
            {
                "name": "down: Stops all containers and tears down the network infrastructure",
                "value": "down",
            },
            {
                "name": "build: Re-compile all source images without starting the contaners",
                "value": "build",
            },
            {
                "name": "stop: like down, but pauses the contaners instead of destorying them, usful if you need some compute resources back but dont want to dealte unsaved work",
                "value": "stop",
            },
            {
                "name": "start: used with stop, start restarts any ALLREADY EXSISTING contaners (so if you ran down instead of stop, it will NOT work. run up instead)",
                "value": "stop",
            },
        ],
    ).ask()


    return action
def make_args():
    if use_sudo:
        cmdline.insert(0, "sudo")
    if use_dev:
        cmdline.extend(("-f", "docker-compose.dev.yaml"))
if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-s",
        "--skip",
        action="store_true",
        help="uses the recomended settings and skips straight to asking what command to run (defaults are chdir as a safety meshure, run as user, and build for dev envroment.)",
    )
    args = parser.parse_args()
    while True:
        if args.skip:
            # Apply fast defaults without asking questions. if you want to change defults, just change the bool, its pritty self explanitory. also if you want to change should_chdir, which you shouldnt, get rid of chdir_wikijump in skip
            use_sudo = False
            chdir_wikijump()
            use_dev = True
        else:
            use_sudo = is_sudo()
            should_chdir()
            use_dev = is_dev()
        cmdline = [
        "docker",
        "compose",
        "-p",
        "wikijump",
        "-f",
        "docker-compose.yaml",
        ]
        make_args()
        docker_action = get_docker_action()
        cmdline.extend(docker_action.split())
        print(" ".join(cmdline))
        yn =  questionary.select(
            "please comfirm that this is the command you wish to use",
            choices=[
                "yes",
                "no",
        ],
        ).ask()
        if yn == "yes":
            break
    print(" ".join(cmdline))
    os.execvp(cmdline[0], cmdline)
