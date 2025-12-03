import json
import math
import sys
from PIL import Image, ImageDraw

def get_ball_directions(ball_count):
    delta_angle = 2.0 * math.pi / ball_count
    angle = 0.0
    directions = []
    for _ in range(ball_count):
        x = math.cos(angle)
        y = math.sin(angle)
        directions.append((x, y))
        angle += delta_angle
    return directions

def get_ball_positions(origin, ball_directions, ball_speed, t):
    distance = ball_speed * t
    positions = []
    for direction in ball_directions:
        x = origin[0] + distance * direction[0]
        y = origin[1] + distance * direction[1]
        positions.append((x, y))
    return positions

def create_frame(image_size, ball_positions, ball_radius, ball_color):
    img = Image.new("RGBA", (image_size, image_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    for x, y in ball_positions:
        x1 = int(x - ball_radius)
        y1 = int(y - ball_radius)
        x2 = int(x + ball_radius)
        y2 = int(y + ball_radius)
        draw.ellipse([x1, y1, x2, y2], ball_color)
    return img

def gen_explosion_frames(config_json):
    ball_radius = config_json["ball_radius"]
    ball_count = config_json["ball_count"]
    ball_color = config_json["ball_color"]
    ball_speed = config_json["ball_speed"]
    duration = config_json["duration"]
    frames_per_second = config_json["frames_per_second"]
    frame_duration = 1.0 / float(frames_per_second)
    frame_count = int(duration * frames_per_second)
    ball_directions = get_ball_directions(ball_count)
    image_size = (ball_speed * duration + ball_radius) * 2
    delta_alpha = float(255) / frame_count
    origin = [image_size / 2.0, image_size / 2.0]

    t = 0.0
    alpha = float(ball_color[3])
    images = []
    for _ in range(frame_count):
        ball_positions = get_ball_positions(origin, ball_directions, ball_speed, t)
        color = (ball_color[0], ball_color[1], ball_color[2], int(alpha))
        img = create_frame(int(image_size), ball_positions, ball_radius, color)
        images.append(img)
        t += frame_duration
        alpha -= delta_alpha
        if alpha < 0.0:
            alpha = 0.0

    return images

def merge_frames(images):
    height = max(img.height for img in images)
    width = sum(img.width for img in images)
    image = Image.new("RGBA", (width, height), (0, 0, 0, 0))

    current_x = 0
    for img in images:
        image.paste(img, (current_x, 0))
        current_x += img.width

    return image

def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} config_file output_file")
        exit(1)

    config_file = sys.argv[1]
    output_file = sys.argv[2]

    with open(config_file) as f:
        config_json = json.load(f)

    images = gen_explosion_frames(config_json)
    final_img = merge_frames(images)
    final_img.save(output_file)

if __name__ == "__main__":
    main()
