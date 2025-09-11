import requests
from requests.auth import HTTPBasicAuth

# Your credentials
username = "mannheim"
password = "Exc3llence!"

complete_prefs_data = {
  "student_prefs": [
    {
      "Lars": ["Doktor Sindre", "Doktor Fredrik"],
      "Ljubo": ["Doktor Fredrik", "Doktor Sindre"]
    }
  ],
  "initial_allocations": [
    {
      "Lars": "Doktor Fredrik",
      "Ljubo": "Doktor Sindre"
    }
  ]
}

new_prefs_data = {
  "student_prefs": [
    {
      "Lea": [
        "Public2"
      ],
      "Mia": [
        "Private",
        "Public1"
      ],
      "Kai": [
        "Public1",
        "Private"
      ]
    }
  ],
  "initial_allocations": [
    {
      "Lea": "Private",
      "Mia": "Public1",
      "Kai": "Public2"
    }
  ]
}

response = requests.post(
    "https://api.matchingtools.org/ttc/demo",  # Your URL
    json=new_prefs_data,
    headers={"Content-Type": "application/json"},
    verify=False,
    auth=(username, password)
)

print(f"Status: {response.status_code}")
print(f"Response: {response.text}")
