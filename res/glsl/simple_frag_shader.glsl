#version 330

uniform bool useColor;
uniform bool useTexColor;
uniform vec4 color;
uniform vec4 texColor;
uniform sampler2D texUnit;
uniform float alpha;
in vec2 texCoord;
out vec4 fragColor;

void main()
{
    vec4 tmpColor;
    if(useColor)
    {
        tmpColor = color;
    }
    else
    {
        if(useTexColor)
        {
            tmpColor = texture(texUnit, texCoord) * texColor;
        }
        else
        {
            tmpColor = texture(texUnit, texCoord);
        }
    }

    fragColor = vec4(tmpColor.rgb, alpha * tmpColor.a);
}
