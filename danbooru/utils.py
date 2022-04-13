import datetime
import dateutil.parser

def parse_timestamp(timestamp: str) -> datetime.datetime:
    """
    Parse a timestamp string from Danbooru into a datetime object.
    """
    return dateutil.parser.parse(timestamp)