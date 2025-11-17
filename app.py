from flask import Flask, send_from_directory

app = Flask(__name__, static_folder='assets', static_url_path='/assets')

@app.route('/')
def serve_index():
    return send_from_directory('.', 'index.html')

@app.route('/assets/<path:filename>')
def serve_assets(filename):
    return send_from_directory(app.static_folder, filename)

if __name__ == '__main__':
    app.run()
