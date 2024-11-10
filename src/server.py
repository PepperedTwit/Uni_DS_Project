from flask import Flask, request, jsonify
import socket
import re
import requests
from playwright.sync_api import sync_playwright

app = Flask(__name__)

# ----------------------------------------------------------------------------- #
# ---------------------------------- Functions -------------------------------- #
# ----------------------------------------------------------------------------- #

def get_html(url: str) -> tuple[str, int | None]:
    browser = None
    latest = None, None

    with sync_playwright() as p:
        try:
            browser = p.chromium.launch(headless=False)
            context = browser.new_context(
                user_agent=(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                    "AppleWebKit/537.36 (KHTML, like Gecko) "
                    "Chrome/91.0.4472.124 Safari/537.36"
                )
            )
            page = context.new_page()

            page.goto(url, timeout=60000)
            page.get_by_role("link", name="Financials & Documents").click()
            page.wait_for_load_state('networkidle')

            ais_set = page.get_by_title("View Annual Information Statement")

            for i in range(ais_set.count()):
                ais = ais_set.nth(i)
                title = ais.get_attribute("title")
                year_match = re.search(r"\d{4}", title)

                if year_match:
                    year = int(year_match.group(0))
                    if latest[0] is None or year > latest[1]:
                        latest = ais, year

            if latest[0] is None:
                return "Error: No AIS found", None

            latest[0].click()
            page.wait_for_load_state('networkidle')

            return page.content(), latest[1]

        except Exception as e:
            return f"Error: {e}", None

        finally:
            if browser:
                browser.close()

def get_dataset_resources(sql: str) -> dict:
    base_url = "https://data.gov.au/data/api/3/action/datastore_search_sql"
    encoded_sql = requests.utils.quote(sql)
    url = f"{base_url}?sql={encoded_sql}"

    response = requests.get(url)

    if response.status_code == 200:
        return response.json()
    else:
        print(f"Error accessing API: Status code {response.status_code}")
        return {}

# ----------------------------------------------------------------------------- #
# ------------------------------- Function Calls ------------------------------ #
# ----------------------------------------------------------------------------- #

# Call get_dataset_resources function
dataset_resources = get_dataset_resources("""
SELECT *
FROM "eb1e6be4-5b13-4feb-b28e-388bf7c26f93"
WHERE "Charity_Legal_Name" LIKE '%Land Council%'
LIMIT 1000
""")

# ----------------------------------------------------------------------------- #
# ------------------------------------ GET ------------------------------------ #
# ----------------------------------------------------------------------------- #

# Example endpoint: Add two numbers
@app.route('/add', methods=['GET'])
def add():
    # Get parameters from the request
    a = request.args.get('a', type=float)
    b = request.args.get('b', type=float)
    result = a + b
    return jsonify({"result": result})

# New endpoint: Return "Hello, world!"
@app.route('/hello', methods=['GET'])
def hello():
    return jsonify({"message": "Hello, world!"})

# ----------------------------------------------------------------------------- #
# ------------------------------------ POST ----------------------------------- #
# ----------------------------------------------------------------------------- #

# Example endpoint: Echo a message
@app.route('/echo', methods=['POST'])
def echo():
    data = request.json
    return jsonify({"echo": data["message"]})

# ----------------------------------------------------------------------------- #
# --------------------------------- Utilities --------------------------------- #
# ----------------------------------------------------------------------------- #

def get_local_ip():
    # Use socket to find the local IP address
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        # Connect to an external server to get the local IP
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
    except Exception:
        ip = "127.0.0.1"
    finally:
        s.close()
    return ip

# ----------------------------------------------------------------------------- #
# ------------------------------------ Main ----------------------------------- #
# ----------------------------------------------------------------------------- #

if __name__ == '__main__':
    host_ip = get_local_ip()
    print(f"Starting server on {host_ip}:5000")
    app.run(host=host_ip, port=5000)
