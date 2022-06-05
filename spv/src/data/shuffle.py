def unit_swizzels(r):
	for i in range(0, r):
		c = ""
		if i == 0:
			c = "x"
		elif i == 1:
			c = "y"
		elif i == 2:
			c = "z"
		elif i == 3:
			c = "w"
		print("{}, {},".format(c, i))

unit_swizzels(2)
print("")
unit_swizzels(3)
print("")
unit_swizzels(4)
print("")

def vec2_swizzels(r):
	for i in range(0, r):
		for j in range(0, r):
			c0 = ""
			if i == 0:
				c0 = "x"
			elif i == 1:
				c0 = "y"
			elif i == 2:
				c0 = "z"
			elif i == 3:
				c0 = "w"
			c1 = ""
			if j == 0:
				c1 = "x"
			elif j == 1:
				c1 = "y"
			elif j == 2:
				c1 = "z"
			elif j == 3:
				c1 = "w"
			print("{}{}, {}, {},".format(c0, c1, i, j))

vec2_swizzels(2)
print("")
vec2_swizzels(3)
print("")
vec2_swizzels(4)
print("")

def vec3_swizzels(r):
	for i in range(0, r):
		for j in range(0, r):
			for k in range(0, r):
				c0 = ""
				if i == 0:
					c0 = "x"
				elif i == 1:
					c0 = "y"
				elif i == 2:
					c0 = "z"
				elif i == 3:
					c0 = "w"
				c1 = ""
				if j == 0:
					c1 = "x"
				elif j == 1:
					c1 = "y"
				elif j == 2:
					c1 = "z"
				elif j == 3:
					c1 = "w"
				c2 = ""
				if k == 0:
					c2 = "x"
				elif k == 1:
					c2 = "y"
				elif k == 2:
					c2 = "z"
				elif k == 3:
					c2 = "w"
				print("{}{}{}, {}, {}, {},".format(c0, c1, c2, i, j, k))

vec3_swizzels(2)
print("")
vec3_swizzels(3)
print("")
vec3_swizzels(4)
print("")

def vec4_swizzels(r):
	for i in range(0, r):
		for j in range(0, r):
			for k in range(0, r):
				for l in range(0, r):
					c0 = ""
					if i == 0:
						c0 = "x"
					elif i == 1:
						c0 = "y"
					elif i == 2:
						c0 = "z"
					elif i == 3:
						c0 = "w"
					c1 = ""
					if j == 0:
						c1 = "x"
					elif j == 1:
						c1 = "y"
					elif j == 2:
						c1 = "z"
					elif j == 3:
						c1 = "w"
					c2 = ""
					if k == 0:
						c2 = "x"
					elif k == 1:
						c2 = "y"
					elif k == 2:
						c2 = "z"
					elif k == 3:
						c2 = "w"
					c3 = ""
					if l == 0:
						c3 = "x"
					elif l == 1:
						c3 = "y"
					elif l == 2:
						c3 = "z"
					elif l == 3:
						c3 = "w"
					print("{}{}{}{}, {}, {}, {}, {},".format(c0, c1, c2, c3, i, j, k, l))	

vec4_swizzels(2)
print("")
vec4_swizzels(3)
print("")
vec4_swizzels(4)
print("")