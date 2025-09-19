const float ATLAS_SIZE_X = 3.0; 
const float ATLAS_SIZE_Y = 10.0;

const vec3 normals[6] = vec3[6](
    vec3(-1.0, 0.0, 0.0), 
    vec3(1.0, 0.0, 0.0), 
    vec3(0.0,-1.0, 0.0), 
    vec3(0.0, 1.0, 0.0), 
    vec3(0.0, 0.0,-1.0), 
    vec3(0.0, 0.0, 1.0));

const vec4 ao_values = vec4(1.0,0.7,0.5,0.15);

const float secs_in_day = 86400.0;
const float pi = 3.141592;

float day_factor(float seconds) {
    const float night_start = 23.0 * 3600.0;
    const float night_end = 4.0 * 3600.0;
    const float day_start = 10.0 * 3600.0;
    const float day_end = 18.0 * 3600.0;

    if (seconds >= night_end && seconds < day_start) {
        return (seconds - night_end) / (day_start - night_end);
    }
    else if (seconds >= day_start && seconds <= day_end) {
        return 1.0;
    }
    else if (seconds > day_end && seconds < night_start) {
        return 1.0 - (seconds - day_end) / (night_start - day_end);
    }

    return 0.0;
}

mat4 rotate_y(float angle) {
    float c = cos(angle);
    float s = sin(angle);

    return mat4(
        c, 0.0, s, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -s, 0.0, c, 0.0,
        0.0, 0.0, 0.0, 1.0);
}