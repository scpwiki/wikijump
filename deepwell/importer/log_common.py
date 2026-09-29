import logging
import sys
from typing import Final

LOG_FORMAT: Final[str] = "[%(levelname)s] %(asctime)s %(message)s"
LOG_DATE_FORMAT: Final[str] = "%Y/%m/%d %H:%M:%S"


def setup_logging(logger: logging.Logger, debug: bool = False) -> None:
    log_fmt = logging.Formatter(LOG_FORMAT, datefmt=LOG_DATE_FORMAT)

    log_stdout = logging.StreamHandler(sys.stdout)
    log_stdout.setFormatter(log_fmt)

    logger.setLevel(level=logging.DEBUG if debug else logging.INFO)
    logger.addHandler(log_stdout)
