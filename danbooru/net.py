import requests
import time

session = requests.Session()
session.proxies = {
    'http': 'socks5://localhost:1080',
    'https': 'socks5://localhost:1080'
}

def get(url, **kwargs):
    if not url.startswith('https://'):
        url = 'https://danbooru.donmai.us/' + url
    response = session.get(url, **kwargs)
    delay = 0.5
    while response.status_code == 429 or response.status_code>=500:
        print(f"Error {response.status_code}, waiting {delay}...")
        time.sleep(delay)
        delay *= 2
        response = session.get(url, **kwargs)
    return response
