/* PrismJS 1.23.0
https://prismjs.com/download.html#themes=prism&languages=markup+css+clike+javascript+abnf+actionscript+apl+arduino+asciidoc+aspnet+autohotkey+bash+basic+batch+bnf+brainfuck+brightscript+c+csharp+cpp+clojure+cobol+coffeescript+crystal+css-extras+csv+d+dart+diff+docker+ebnf+editorconfig+elixir+elm+erlang+fsharp+flow+fortran+git+glsl+go+graphql+haskell+hcl+hlsl+http+hpkp+hsts+ignore+ini+java+javadoc+javadoclike+jsdoc+js-extras+json+json5+jsonp+jsstacktrace+julia+kotlin+latex+less+lisp+log+lua+makefile+markdown+markup-templating+matlab+nasm+nginx+nim+objectivec+ocaml+opencl+pascal+perl+php+phpdoc+php-extras+plsql+powershell+purescript+python+qml+r+jsx+tsx+regex+rest+ruby+rust+sass+scss+scala+scheme+smalltalk+smarty+sql+stylus+swift+toml+typescript+v+vim+wasm+yaml+zig&plugins=autolinker+custom-class+highlight-keywords */

export function prismBase(Prism) {
	Prism.languages.markup = {
		'comment': /<!--[\s\S]*?-->/,
		'prolog': /<\?[\s\S]+?\?>/,
		'doctype': {
			// https://www.w3.org/TR/xml/#NT-doctypedecl
			pattern: /<!DOCTYPE(?:[^>"'[\]]|"[^"]*"|'[^']*')+(?:\[(?:[^<"'\]]|"[^"]*"|'[^']*'|<(?!!--)|<!--(?:[^-]|-(?!->))*-->)*\]\s*)?>/i,
			greedy: true,
			inside: {
				'internal-subset': {
					pattern: /(\[)[\s\S]+(?=\]>$)/,
					lookbehind: true,
					greedy: true,
					inside: null // see below
				},
				'string': {
					pattern: /"[^"]*"|'[^']*'/,
					greedy: true
				},
				'punctuation': /^<!|>$|[[\]]/,
				'doctype-tag': /^DOCTYPE/,
				'name': /[^\s<>'"]+/
			}
		},
		'cdata': /<!\[CDATA\[[\s\S]*?]]>/i,
		'tag': {
			pattern: /<\/?(?!\d)[^\s>\/=$<%]+(?:\s(?:\s*[^\s>\/=]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s'">=]+(?=[\s>]))|(?=[\s/>])))+)?\s*\/?>/,
			greedy: true,
			inside: {
				'tag': {
					pattern: /^<\/?[^\s>\/]+/,
					inside: {
						'punctuation': /^<\/?/,
						'namespace': /^[^\s>\/:]+:/
					}
				},
				'special-attr': [],
				'attr-value': {
					pattern: /=\s*(?:"[^"]*"|'[^']*'|[^\s'">=]+)/,
					inside: {
						'punctuation': [
							{
								pattern: /^=/,
								alias: 'attr-equals'
							},
							/"|'/
						]
					}
				},
				'punctuation': /\/?>/,
				'attr-name': {
					pattern: /[^\s>\/]+/,
					inside: {
						'namespace': /^[^\s>\/:]+:/
					}
				}

			}
		},
		'entity': [
			{
				pattern: /&[\da-z]{1,8};/i,
				alias: 'named-entity'
			},
			/&#x?[\da-f]{1,8};/i
		]
	};

	Prism.languages.markup['tag'].inside['attr-value'].inside['entity'] =
		Prism.languages.markup['entity'];
	Prism.languages.markup['doctype'].inside['internal-subset'].inside = Prism.languages.markup;

	// Plugin to make entity title show the real entity, idea by Roman Komarov
	Prism.hooks.add('wrap', function (env) {

		if (env.type === 'entity') {
			env.attributes['title'] = env.content.replace(/&amp;/, '&');
		}
	});

	Object.defineProperty(Prism.languages.markup.tag, 'addInlined', {
		/**
		 * Adds an inlined language to markup.
		 *
		 * An example of an inlined language is CSS with `<style>` tags.
		 *
		 * @param {string} tagName The name of the tag that contains the inlined language. This name will be treated as
		 * case insensitive.
		 * @param {string} lang The language key.
		 * @example
		 * addInlined('style', 'css');
		 */
		value: function addInlined(tagName, lang) {
			var includedCdataInside = {};
			includedCdataInside['language-' + lang] = {
				pattern: /(^<!\[CDATA\[)[\s\S]+?(?=\]\]>$)/i,
				lookbehind: true,
				inside: Prism.languages[lang]
			};
			includedCdataInside['cdata'] = /^<!\[CDATA\[|\]\]>$/i;

			var inside = {
				'included-cdata': {
					pattern: /<!\[CDATA\[[\s\S]*?\]\]>/i,
					inside: includedCdataInside
				}
			};
			inside['language-' + lang] = {
				pattern: /[\s\S]+/,
				inside: Prism.languages[lang]
			};

			var def = {};
			def[tagName] = {
				pattern: RegExp(/(<__[^>]*>)(?:<!\[CDATA\[(?:[^\]]|\](?!\]>))*\]\]>|(?!<!\[CDATA\[)[\s\S])*?(?=<\/__>)/.source.replace(/__/g, function () { return tagName; }), 'i'),
				lookbehind: true,
				greedy: true,
				inside: inside
			};

			Prism.languages.insertBefore('markup', 'cdata', def);
		}
	});
	Object.defineProperty(Prism.languages.markup.tag, 'addAttribute', {
		/**
		 * Adds an pattern to highlight languages embedded in HTML attributes.
		 *
		 * An example of an inlined language is CSS with `style` attributes.
		 *
		 * @param {string} attrName The name of the tag that contains the inlined language. This name will be treated as
		 * case insensitive.
		 * @param {string} lang The language key.
		 * @example
		 * addAttribute('style', 'css');
		 */
		value: function (attrName, lang) {
			Prism.languages.markup.tag.inside['special-attr'].push({
				pattern: RegExp(
					/(^|["'\s])/.source + '(?:' + attrName + ')' + /\s*=\s*(?:"[^"]*"|'[^']*'|[^\s'">=]+(?=[\s>]))/.source,
					'i'
				),
				lookbehind: true,
				inside: {
					'attr-name': /^[^\s=]+/,
					'attr-value': {
						pattern: /=[\s\S]+/,
						inside: {
							'value': {
								pattern: /(=\s*(["']|(?!["'])))\S[\s\S]*(?=\2$)/,
								lookbehind: true,
								alias: [lang, 'language-' + lang],
								inside: Prism.languages[lang]
							},
							'punctuation': [
								{
									pattern: /^=/,
									alias: 'attr-equals'
								},
								/"|'/
							]
						}
					}
				}
			});
		}
	});

	Prism.languages.html = Prism.languages.markup;
	Prism.languages.mathml = Prism.languages.markup;
	Prism.languages.svg = Prism.languages.markup;

	Prism.languages.xml = Prism.languages.extend('markup', {});
	Prism.languages.ssml = Prism.languages.xml;
	Prism.languages.atom = Prism.languages.xml;
	Prism.languages.rss = Prism.languages.xml;

	(function (Prism) {

		var string = /("|')(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/;

		Prism.languages.css = {
			'comment': /\/\*[\s\S]*?\*\//,
			'atrule': {
				pattern: /@[\w-](?:[^;{\s]|\s+(?![\s{]))*(?:;|(?=\s*\{))/,
				inside: {
					'rule': /^@[\w-]+/,
					'selector-function-argument': {
						pattern: /(\bselector\s*\(\s*(?![\s)]))(?:[^()\s]|\s+(?![\s)])|\((?:[^()]|\([^()]*\))*\))+(?=\s*\))/,
						lookbehind: true,
						alias: 'selector'
					},
					'keyword': {
						pattern: /(^|[^\w-])(?:and|not|only|or)(?![\w-])/,
						lookbehind: true
					}
					// See rest below
				}
			},
			'url': {
				// https://drafts.csswg.org/css-values-3/#urls
				pattern: RegExp('\\burl\\((?:' + string.source + '|' + /(?:[^\\\r\n()"']|\\[\s\S])*/.source + ')\\)', 'i'),
				greedy: true,
				inside: {
					'function': /^url/i,
					'punctuation': /^\(|\)$/,
					'string': {
						pattern: RegExp('^' + string.source + '$'),
						alias: 'url'
					}
				}
			},
			'selector': RegExp('[^{}\\s](?:[^{};"\'\\s]|\\s+(?![\\s{])|' + string.source + ')*(?=\\s*\\{)'),
			'string': {
				pattern: string,
				greedy: true
			},
			'property': /(?!\s)[-_a-z\xA0-\uFFFF](?:(?!\s)[-\w\xA0-\uFFFF])*(?=\s*:)/i,
			'important': /!important\b/i,
			'function': /[-a-z0-9]+(?=\()/i,
			'punctuation': /[(){};:,]/
		};

		Prism.languages.css['atrule'].inside.rest = Prism.languages.css;

		var markup = Prism.languages.markup;
		if (markup) {
			markup.tag.addInlined('style', 'css');
			markup.tag.addAttribute('style', 'css');
		}

	}(Prism));

	Prism.languages.clike = {
		'comment': [
			{
				pattern: /(^|[^\\])\/\*[\s\S]*?(?:\*\/|$)/,
				lookbehind: true,
				greedy: true
			},
			{
				pattern: /(^|[^\\:])\/\/.*/,
				lookbehind: true,
				greedy: true
			}
		],
		'string': {
			pattern: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/,
			greedy: true
		},
		'class-name': {
			pattern: /(\b(?:class|interface|extends|implements|trait|instanceof|new)\s+|\bcatch\s+\()[\w.\\]+/i,
			lookbehind: true,
			inside: {
				'punctuation': /[.\\]/
			}
		},
		'keyword': /\b(?:if|else|while|do|for|return|in|instanceof|function|new|try|throw|catch|finally|null|break|continue)\b/,
		'boolean': /\b(?:true|false)\b/,
		'function': /\w+(?=\()/,
		'number': /\b0x[\da-f]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?/i,
		'operator': /[<>]=?|[!=]=?=?|--?|\+\+?|&&?|\|\|?|[?*/~^%]/,
		'punctuation': /[{}[\];(),.:]/
	};

	Prism.languages.javascript = Prism.languages.extend('clike', {
		'class-name': [
			Prism.languages.clike['class-name'],
			{
				pattern: /(^|[^$\w\xA0-\uFFFF])(?!\s)[_$A-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\.(?:prototype|constructor))/,
				lookbehind: true
			}
		],
		'keyword': [
			{
				pattern: /((?:^|})\s*)catch\b/,
				lookbehind: true
			},
			{
				pattern: /(^|[^.]|\.\.\.\s*)\b(?:as|async(?=\s*(?:function\b|\(|[$\w\xA0-\uFFFF]|$))|await|break|case|class|const|continue|debugger|default|delete|do|else|enum|export|extends|finally(?=\s*(?:\{|$))|for|from(?=\s*(?:['"]|$))|function|(?:get|set)(?=\s*(?:[#\[$\w\xA0-\uFFFF]|$))|if|implements|import|in|instanceof|interface|let|new|null|of|package|private|protected|public|return|static|super|switch|this|throw|try|typeof|undefined|var|void|while|with|yield)\b/,
				lookbehind: true
			},
		],
		// Allow for all non-ASCII characters (See http://stackoverflow.com/a/2008444)
		'function': /#?(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\s*(?:\.\s*(?:apply|bind|call)\s*)?\()/,
		'number': /\b(?:(?:0[xX](?:[\dA-Fa-f](?:_[\dA-Fa-f])?)+|0[bB](?:[01](?:_[01])?)+|0[oO](?:[0-7](?:_[0-7])?)+)n?|(?:\d(?:_\d)?)+n|NaN|Infinity)\b|(?:\b(?:\d(?:_\d)?)+\.?(?:\d(?:_\d)?)*|\B\.(?:\d(?:_\d)?)+)(?:[Ee][+-]?(?:\d(?:_\d)?)+)?/,
		'operator': /--|\+\+|\*\*=?|=>|&&=?|\|\|=?|[!=]==|<<=?|>>>?=?|[-+*/%&|^!=<>]=?|\.{3}|\?\?=?|\?\.?|[~:]/
	});

	Prism.languages.javascript['class-name'][0].pattern = /(\b(?:class|interface|extends|implements|instanceof|new)\s+)[\w.\\]+/;

	Prism.languages.insertBefore('javascript', 'keyword', {
		'regex': {
			pattern: /((?:^|[^$\w\xA0-\uFFFF."'\])\s]|\b(?:return|yield))\s*)\/(?:\[(?:[^\]\\\r\n]|\\.)*]|\\.|[^/\\\[\r\n])+\/[gimyus]{0,6}(?=(?:\s|\/\*(?:[^*]|\*(?!\/))*\*\/)*(?:$|[\r\n,.;:})\]]|\/\/))/,
			lookbehind: true,
			greedy: true,
			inside: {
				'regex-source': {
					pattern: /^(\/)[\s\S]+(?=\/[a-z]*$)/,
					lookbehind: true,
					alias: 'language-regex',
					inside: Prism.languages.regex
				},
				'regex-flags': /[a-z]+$/,
				'regex-delimiter': /^\/|\/$/
			}
		},
		// This must be declared before keyword because we use "function" inside the look-forward
		'function-variable': {
			pattern: /#?(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\s*[=:]\s*(?:async\s*)?(?:\bfunction\b|(?:\((?:[^()]|\([^()]*\))*\)|(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*)\s*=>))/,
			alias: 'function'
		},
		'parameter': [
			{
				pattern: /(function(?:\s+(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*)?\s*\(\s*)(?!\s)(?:[^()\s]|\s+(?![\s)])|\([^()]*\))+(?=\s*\))/,
				lookbehind: true,
				inside: Prism.languages.javascript
			},
			{
				pattern: /(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\s*=>)/i,
				inside: Prism.languages.javascript
			},
			{
				pattern: /(\(\s*)(?!\s)(?:[^()\s]|\s+(?![\s)])|\([^()]*\))+(?=\s*\)\s*=>)/,
				lookbehind: true,
				inside: Prism.languages.javascript
			},
			{
				pattern: /((?:\b|\s|^)(?!(?:as|async|await|break|case|catch|class|const|continue|debugger|default|delete|do|else|enum|export|extends|finally|for|from|function|get|if|implements|import|in|instanceof|interface|let|new|null|of|package|private|protected|public|return|set|static|super|switch|this|throw|try|typeof|undefined|var|void|while|with|yield)(?![$\w\xA0-\uFFFF]))(?:(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*\s*)\(\s*|\]\s*\(\s*)(?!\s)(?:[^()\s]|\s+(?![\s)])|\([^()]*\))+(?=\s*\)\s*\{)/,
				lookbehind: true,
				inside: Prism.languages.javascript
			}
		],
		'constant': /\b[A-Z](?:[A-Z_]|\dx?)*\b/
	});

	Prism.languages.insertBefore('javascript', 'string', {
		'hashbang': {
			pattern: /^#!.*/,
			greedy: true,
			alias: 'comment'
		},
		'template-string': {
			pattern: /`(?:\\[\s\S]|\${(?:[^{}]|{(?:[^{}]|{[^}]*})*})+}|(?!\${)[^\\`])*`/,
			greedy: true,
			inside: {
				'template-punctuation': {
					pattern: /^`|`$/,
					alias: 'string'
				},
				'interpolation': {
					pattern: /((?:^|[^\\])(?:\\{2})*)\${(?:[^{}]|{(?:[^{}]|{[^}]*})*})+}/,
					lookbehind: true,
					inside: {
						'interpolation-punctuation': {
							pattern: /^\${|}$/,
							alias: 'punctuation'
						},
						rest: Prism.languages.javascript
					}
				},
				'string': /[\s\S]+/
			}
		}
	});

	if (Prism.languages.markup) {
		Prism.languages.markup.tag.addInlined('script', 'javascript');

		// add attribute support for all DOM events.
		// https://developer.mozilla.org/en-US/docs/Web/Events#Standard_events
		Prism.languages.markup.tag.addAttribute(
			/on(?:abort|blur|change|click|composition(?:end|start|update)|dblclick|error|focus(?:in|out)?|key(?:down|up)|load|mouse(?:down|enter|leave|move|out|over|up)|reset|resize|scroll|select|slotchange|submit|unload|wheel)/.source,
			'javascript'
		);
	}

	Prism.languages.js = Prism.languages.javascript;

	(function (Prism) {

		var coreRules = '(?:ALPHA|BIT|CHAR|CR|CRLF|CTL|DIGIT|DQUOTE|HEXDIG|HTAB|LF|LWSP|OCTET|SP|VCHAR|WSP)';

		Prism.languages.abnf = {
			'comment': /;.*/,
			'string': {
				pattern: /(?:%[is])?"[^"\n\r]*"/,
				greedy: true,
				inside: {
					'punctuation': /^%[is]/
				}
			},
			'range': {
				pattern: /%(?:b[01]+-[01]+|d\d+-\d+|x[A-F\d]+-[A-F\d]+)/i,
				alias: 'number'
			},
			'terminal': {
				pattern: /%(?:b[01]+(?:\.[01]+)*|d\d+(?:\.\d+)*|x[A-F\d]+(?:\.[A-F\d]+)*)/i,
				alias: 'number'
			},
			'repetition': {
				pattern: /(^|[^\w-])(?:\d*\*\d*|\d+)/,
				lookbehind: true,
				alias: 'operator'
			},
			'definition': {
				pattern: /(^[ \t]*)(?:[a-z][\w-]*|<[^>\r\n]*>)(?=\s*=)/m,
				lookbehind: true,
				alias: 'keyword',
				inside: {
					'punctuation': /<|>/
				}
			},
			'core-rule': {
				pattern: RegExp('(?:(^|[^<\\w-])' + coreRules + '|<' + coreRules + '>)(?![\\w-])', 'i'),
				lookbehind: true,
				alias: ['rule', 'constant'],
				inside: {
					'punctuation': /<|>/
				}
			},
			'rule': {
				pattern: /(^|[^<\w-])[a-z][\w-]*|<[^>\r\n]*>/i,
				lookbehind: true,
				inside: {
					'punctuation': /<|>/
				}
			},
			'operator': /=\/?|\//,
			'punctuation': /[()\[\]]/
		};

	}(Prism));

	Prism.languages.actionscript = Prism.languages.extend('javascript', {
		'keyword': /\b(?:as|break|case|catch|class|const|default|delete|do|else|extends|finally|for|function|if|implements|import|in|instanceof|interface|internal|is|native|new|null|package|private|protected|public|return|super|switch|this|throw|try|typeof|use|var|void|while|with|dynamic|each|final|get|include|namespace|native|override|set|static)\b/,
		'operator': /\+\+|--|(?:[+\-*\/%^]|&&?|\|\|?|<<?|>>?>?|[!=]=?)=?|[~?@]/
	});
	Prism.languages.actionscript['class-name'].alias = 'function';

	if (Prism.languages.markup) {
		Prism.languages.insertBefore('actionscript', 'string', {
			'xml': {
				pattern: /(^|[^.])<\/?\w+(?:\s+[^\s>\/=]+=("|')(?:\\[\s\S]|(?!\2)[^\\])*\2)*\s*\/?>/,
				lookbehind: true,
				inside: Prism.languages.markup
			}
		});
	}
	;
	Prism.languages.apl = {
		'comment': /(?:⍝|#[! ]).*$/m,
		'string': {
			pattern: /'(?:[^'\r\n]|'')*'/,
			greedy: true
		},
		'number': /¯?(?:\d*\.?\b\d+(?:e[+¯]?\d+)?|¯|∞)(?:j¯?(?:(?:\d+(?:\.\d+)?|\.\d+)(?:e[+¯]?\d+)?|¯|∞))?/i,
		'statement': /:[A-Z][a-z][A-Za-z]*\b/,
		'system-function': {
			pattern: /⎕[A-Z]+/i,
			alias: 'function'
		},
		'constant': /[⍬⌾#⎕⍞]/,
		'function': /[-+×÷⌈⌊∣|⍳⍸?*⍟○!⌹<≤=>≥≠≡≢∊⍷∪∩~∨∧⍱⍲⍴,⍪⌽⊖⍉↑↓⊂⊃⊆⊇⌷⍋⍒⊤⊥⍕⍎⊣⊢⍁⍂≈⍯↗¤→]/,
		'monadic-operator': {
			pattern: /[\\\/⌿⍀¨⍨⌶&∥]/,
			alias: 'operator'
		},
		'dyadic-operator': {
			pattern: /[.⍣⍠⍤∘⌸@⌺⍥]/,
			alias: 'operator'
		},
		'assignment': {
			pattern: /←/,
			alias: 'keyword'
		},
		'punctuation': /[\[;\]()◇⋄]/,
		'dfn': {
			pattern: /[{}⍺⍵⍶⍹∇⍫:]/,
			alias: 'builtin'
		}
	};

	Prism.languages.c = Prism.languages.extend('clike', {
		'comment': {
			pattern: /\/\/(?:[^\r\n\\]|\\(?:\r\n?|\n|(?![\r\n])))*|\/\*[\s\S]*?(?:\*\/|$)/,
			greedy: true
		},
		'class-name': {
			pattern: /(\b(?:enum|struct)\s+(?:__attribute__\s*\(\([\s\S]*?\)\)\s*)?)\w+|\b[a-z]\w*_t\b/,
			lookbehind: true
		},
		'keyword': /\b(?:__attribute__|_Alignas|_Alignof|_Atomic|_Bool|_Complex|_Generic|_Imaginary|_Noreturn|_Static_assert|_Thread_local|asm|typeof|inline|auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|int|long|register|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while)\b/,
		'function': /[a-z_]\w*(?=\s*\()/i,
		'number': /(?:\b0x(?:[\da-f]+(?:\.[\da-f]*)?|\.[\da-f]+)(?:p[+-]?\d+)?|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?)[ful]{0,4}/i,
		'operator': />>=?|<<=?|->|([-+&|:])\1|[?:~]|[-+*/%&|^!=<>]=?/
	});

	Prism.languages.insertBefore('c', 'string', {
		'macro': {
			// allow for multiline macro definitions
			// spaces after the # character compile fine with gcc
			pattern: /(^\s*)#\s*[a-z](?:[^\r\n\\/]|\/(?!\*)|\/\*(?:[^*]|\*(?!\/))*\*\/|\\(?:\r\n|[\s\S]))*/im,
			lookbehind: true,
			greedy: true,
			alias: 'property',
			inside: {
				'string': [
					{
						// highlight the path of the include statement as a string
						pattern: /^(#\s*include\s*)<[^>]+>/,
						lookbehind: true
					},
					Prism.languages.c['string']
				],
				'comment': Prism.languages.c['comment'],
				'macro-name': [
					{
						pattern: /(^#\s*define\s+)\w+\b(?!\()/i,
						lookbehind: true
					},
					{
						pattern: /(^#\s*define\s+)\w+\b(?=\()/i,
						lookbehind: true,
						alias: 'function'
					}
				],
				// highlight macro directives as keywords
				'directive': {
					pattern: /^(#\s*)[a-z]+/,
					lookbehind: true,
					alias: 'keyword'
				},
				'directive-hash': /^#/,
				'punctuation': /##|\\(?=[\r\n])/,
				'expression': {
					pattern: /\S[\s\S]*/,
					inside: Prism.languages.c
				}
			}
		},
		// highlight predefined macros as constants
		'constant': /\b(?:__FILE__|__LINE__|__DATE__|__TIME__|__TIMESTAMP__|__func__|EOF|NULL|SEEK_CUR|SEEK_END|SEEK_SET|stdin|stdout|stderr)\b/
	});

	delete Prism.languages.c['boolean'];

	(function (Prism) {

		var keyword = /\b(?:alignas|alignof|asm|auto|bool|break|case|catch|char|char8_t|char16_t|char32_t|class|compl|concept|const|consteval|constexpr|constinit|const_cast|continue|co_await|co_return|co_yield|decltype|default|delete|do|double|dynamic_cast|else|enum|explicit|export|extern|final|float|for|friend|goto|if|import|inline|int|int8_t|int16_t|int32_t|int64_t|uint8_t|uint16_t|uint32_t|uint64_t|long|module|mutable|namespace|new|noexcept|nullptr|operator|override|private|protected|public|register|reinterpret_cast|requires|return|short|signed|sizeof|static|static_assert|static_cast|struct|switch|template|this|thread_local|throw|try|typedef|typeid|typename|union|unsigned|using|virtual|void|volatile|wchar_t|while)\b/;
		var modName = /\b(?!<keyword>)\w+(?:\s*\.\s*\w+)*\b/.source.replace(/<keyword>/g, function () { return keyword.source; });

		Prism.languages.cpp = Prism.languages.extend('c', {
			'class-name': [
				{
					pattern: RegExp(/(\b(?:class|concept|enum|struct|typename)\s+)(?!<keyword>)\w+/.source
						.replace(/<keyword>/g, function () { return keyword.source; })),
					lookbehind: true
				},
				// This is intended to capture the class name of method implementations like:
				//   void foo::bar() const {}
				// However! The `foo` in the above example could also be a namespace, so we only capture the class name if
				// it starts with an uppercase letter. This approximation should give decent results.
				/\b[A-Z]\w*(?=\s*::\s*\w+\s*\()/,
				// This will capture the class name before destructors like:
				//   Foo::~Foo() {}
				/\b[A-Z_]\w*(?=\s*::\s*~\w+\s*\()/i,
				// This also intends to capture the class name of method implementations but here the class has template
				// parameters, so it can't be a namespace (until C++ adds generic namespaces).
				/\w+(?=\s*<(?:[^<>]|<(?:[^<>]|<[^<>]*>)*>)*>\s*::\s*\w+\s*\()/
			],
			'keyword': keyword,
			'number': {
				pattern: /(?:\b0b[01']+|\b0x(?:[\da-f']+(?:\.[\da-f']*)?|\.[\da-f']+)(?:p[+-]?[\d']+)?|(?:\b[\d']+(?:\.[\d']*)?|\B\.[\d']+)(?:e[+-]?[\d']+)?)[ful]{0,4}/i,
				greedy: true
			},
			'operator': />>=?|<<=?|->|--|\+\+|&&|\|\||[?:~]|<=>|[-+*/%&|^!=<>]=?|\b(?:and|and_eq|bitand|bitor|not|not_eq|or|or_eq|xor|xor_eq)\b/,
			'boolean': /\b(?:true|false)\b/
		});

		Prism.languages.insertBefore('cpp', 'string', {
			'module': {
				// https://en.cppreference.com/w/cpp/language/modules
				pattern: RegExp(
					/(\b(?:module|import)\s+)/.source +
					'(?:' +
					// header-name
					/"(?:\\(?:\r\n|[\s\S])|[^"\\\r\n])*"|<[^<>\r\n]*>/.source +
					'|' +
					// module name or partition or both
					/<mod-name>(?:\s*:\s*<mod-name>)?|:\s*<mod-name>/.source.replace(/<mod-name>/g, function () { return modName; }) +
					')'
				),
				lookbehind: true,
				greedy: true,
				inside: {
					'string': /^[<"][\s\S]+/,
					'operator': /:/,
					'punctuation': /\./
				}
			},
			'raw-string': {
				pattern: /R"([^()\\ ]{0,16})\([\s\S]*?\)\1"/,
				alias: 'string',
				greedy: true
			}
		});

		Prism.languages.insertBefore('cpp', 'keyword', {
			'generic-function': {
				pattern: /\b[a-z_]\w*\s*<(?:[^<>]|<(?:[^<>])*>)*>(?=\s*\()/i,
				inside: {
					'function': /^\w+/,
					'generic': {
						pattern: /<[\s\S]+/,
						alias: 'class-name',
						inside: Prism.languages.cpp
					}
				}
			}
		});

		Prism.languages.insertBefore('cpp', 'operator', {
			'double-colon': {
				pattern: /::/,
				alias: 'punctuation'
			}
		});

		Prism.languages.insertBefore('cpp', 'class-name', {
			// the base clause is an optional list of parent classes
			// https://en.cppreference.com/w/cpp/language/class
			'base-clause': {
				pattern: /(\b(?:class|struct)\s+\w+\s*:\s*)[^;{}"'\s]+(?:\s+[^;{}"'\s]+)*(?=\s*[;{])/,
				lookbehind: true,
				greedy: true,
				inside: Prism.languages.extend('cpp', {})
			}
		});

		Prism.languages.insertBefore('inside', 'double-colon', {
			// All untokenized words that are not namespaces should be class names
			'class-name': /\b[a-z_]\w*\b(?!\s*::)/i
		}, Prism.languages.cpp['base-clause']);

	}(Prism));

	Prism.languages.arduino = Prism.languages.extend('cpp', {
		'constant': /\b(?:DIGITAL_MESSAGE|FIRMATA_STRING|ANALOG_MESSAGE|REPORT_DIGITAL|REPORT_ANALOG|INPUT_PULLUP|SET_PIN_MODE|INTERNAL2V56|SYSTEM_RESET|LED_BUILTIN|INTERNAL1V1|SYSEX_START|INTERNAL|EXTERNAL|DEFAULT|OUTPUT|INPUT|HIGH|LOW)\b/,
		'keyword': /\b(?:setup|if|else|while|do|for|return|in|instanceof|default|function|loop|goto|switch|case|new|try|throw|catch|finally|null|break|continue|boolean|bool|void|byte|word|string|String|array|int|long|integer|double)\b/,
		'builtin': /\b(?:KeyboardController|MouseController|SoftwareSerial|EthernetServer|EthernetClient|LiquidCrystal|LiquidCrystal_I2C|RobotControl|GSMVoiceCall|EthernetUDP|EsploraTFT|HttpClient|RobotMotor|WiFiClient|GSMScanner|FileSystem|Scheduler|GSMServer|YunClient|YunServer|IPAddress|GSMClient|GSMModem|Keyboard|Ethernet|Console|GSMBand|Esplora|Stepper|Process|WiFiUDP|GSM_SMS|Mailbox|USBHost|Firmata|PImage|Client|Server|GSMPIN|FileIO|Bridge|Serial|EEPROM|Stream|Mouse|Audio|Servo|File|Task|GPRS|WiFi|Wire|TFT|GSM|SPI|SD|runShellCommandAsynchronously|analogWriteResolution|retrieveCallingNumber|printFirmwareVersion|analogReadResolution|sendDigitalPortPair|noListenOnLocalhost|readJoystickButton|setFirmwareVersion|readJoystickSwitch|scrollDisplayRight|getVoiceCallStatus|scrollDisplayLeft|writeMicroseconds|delayMicroseconds|beginTransmission|getSignalStrength|runAsynchronously|getAsynchronously|listenOnLocalhost|getCurrentCarrier|readAccelerometer|messageAvailable|sendDigitalPorts|lineFollowConfig|countryNameWrite|runShellCommand|readStringUntil|rewindDirectory|readTemperature|setClockDivider|readLightSensor|endTransmission|analogReference|detachInterrupt|countryNameRead|attachInterrupt|encryptionType|readBytesUntil|robotNameWrite|readMicrophone|robotNameRead|cityNameWrite|userNameWrite|readJoystickY|readJoystickX|mouseReleased|openNextFile|scanNetworks|noInterrupts|digitalWrite|beginSpeaker|mousePressed|isActionDone|mouseDragged|displayLogos|noAutoscroll|addParameter|remoteNumber|getModifiers|keyboardRead|userNameRead|waitContinue|processInput|parseCommand|printVersion|readNetworks|writeMessage|blinkVersion|cityNameRead|readMessage|setDataMode|parsePacket|isListening|setBitOrder|beginPacket|isDirectory|motorsWrite|drawCompass|digitalRead|clearScreen|serialEvent|rightToLeft|setTextSize|leftToRight|requestFrom|keyReleased|compassRead|analogWrite|interrupts|WiFiServer|disconnect|playMelody|parseFloat|autoscroll|getPINUsed|setPINUsed|setTimeout|sendAnalog|readSlider|analogRead|beginWrite|createChar|motorsStop|keyPressed|tempoWrite|readButton|subnetMask|debugPrint|macAddress|writeGreen|randomSeed|attachGPRS|readString|sendString|remotePort|releaseAll|mouseMoved|background|getXChange|getYChange|answerCall|getResult|voiceCall|endPacket|constrain|getSocket|writeJSON|getButton|available|connected|findUntil|readBytes|exitValue|readGreen|writeBlue|startLoop|isPressed|sendSysex|pauseMode|gatewayIP|setCursor|getOemKey|tuneWrite|noDisplay|loadImage|switchPIN|onRequest|onReceive|changePIN|playFile|noBuffer|parseInt|overflow|checkPIN|knobRead|beginTFT|bitClear|updateIR|bitWrite|position|writeRGB|highByte|writeRed|setSpeed|readBlue|noStroke|remoteIP|transfer|shutdown|hangCall|beginSMS|endWrite|attached|maintain|noCursor|checkReg|checkPUK|shiftOut|isValid|shiftIn|pulseIn|connect|println|localIP|pinMode|getIMEI|display|noBlink|process|getBand|running|beginSD|drawBMP|lowByte|setBand|release|bitRead|prepare|pointTo|readRed|setMode|noFill|remove|listen|stroke|detach|attach|noTone|exists|buffer|height|bitSet|circle|config|cursor|random|IRread|setDNS|endSMS|getKey|micros|millis|begin|print|write|ready|flush|width|isPIN|blink|clear|press|mkdir|rmdir|close|point|yield|image|BSSID|click|delay|read|text|move|peek|beep|rect|line|open|seek|fill|size|turn|stop|home|find|step|tone|sqrt|RSSI|SSID|end|bit|tan|cos|sin|pow|map|abs|max|min|get|run|put)\b/
	});

	(function (Prism) {

		var attributes = {
			pattern: /(^[ \t]*)\[(?!\[)(?:(["'$`])(?:(?!\2)[^\\]|\\.)*\2|\[(?:[^\[\]\\]|\\.)*\]|[^\[\]\\"'$`]|\\.)*\]/m,
			lookbehind: true,
			inside: {
				'quoted': {
					pattern: /([$`])(?:(?!\1)[^\\]|\\.)*\1/,
					inside: {
						'punctuation': /^[$`]|[$`]$/
					}
				},
				'interpreted': {
					pattern: /'(?:[^'\\]|\\.)*'/,
					inside: {
						'punctuation': /^'|'$/
						// See rest below
					}
				},
				'string': /"(?:[^"\\]|\\.)*"/,
				'variable': /\w+(?==)/,
				'punctuation': /^\[|\]$|,/,
				'operator': /=/,
				// The negative look-ahead prevents blank matches
				'attr-value': /(?!^\s+$).+/
			}
		};

		var asciidoc = Prism.languages.asciidoc = {
			'comment-block': {
				pattern: /^(\/{4,})(?:\r?\n|\r)(?:[\s\S]*(?:\r?\n|\r))??\1/m,
				alias: 'comment'
			},
			'table': {
				pattern: /^\|={3,}(?:(?:\r?\n|\r(?!\n)).*)*?(?:\r?\n|\r)\|={3,}$/m,
				inside: {
					'specifiers': {
						pattern: /(?!\|)(?:(?:(?:\d+(?:\.\d+)?|\.\d+)[+*])?(?:[<^>](?:\.[<^>])?|\.[<^>])?[a-z]*)(?=\|)/,
						alias: 'attr-value'
					},
					'punctuation': {
						pattern: /(^|[^\\])[|!]=*/,
						lookbehind: true
					}
					// See rest below
				}
			},

			'passthrough-block': {
				pattern: /^(\+{4,})(?:\r?\n|\r)(?:[\s\S]*(?:\r?\n|\r))??\1$/m,
				inside: {
					'punctuation': /^\++|\++$/
					// See rest below
				}
			},
			// Literal blocks and listing blocks
			'literal-block': {
				pattern: /^(-{4,}|\.{4,})(?:\r?\n|\r)(?:[\s\S]*(?:\r?\n|\r))??\1$/m,
				inside: {
					'punctuation': /^(?:-+|\.+)|(?:-+|\.+)$/
					// See rest below
				}
			},
			// Sidebar blocks, quote blocks, example blocks and open blocks
			'other-block': {
				pattern: /^(--|\*{4,}|_{4,}|={4,})(?:\r?\n|\r)(?:[\s\S]*(?:\r?\n|\r))??\1$/m,
				inside: {
					'punctuation': /^(?:-+|\*+|_+|=+)|(?:-+|\*+|_+|=+)$/
					// See rest below
				}
			},

			// list-punctuation and list-label must appear before indented-block
			'list-punctuation': {
				pattern: /(^[ \t]*)(?:-|\*{1,5}|\.{1,5}|(?:[a-z]|\d+)\.|[xvi]+\))(?= )/im,
				lookbehind: true,
				alias: 'punctuation'
			},
			'list-label': {
				pattern: /(^[ \t]*)[a-z\d].+(?::{2,4}|;;)(?=\s)/im,
				lookbehind: true,
				alias: 'symbol'
			},
			'indented-block': {
				pattern: /((\r?\n|\r)\2)([ \t]+)\S.*(?:(?:\r?\n|\r)\3.+)*(?=\2{2}|$)/,
				lookbehind: true
			},

			'comment': /^\/\/.*/m,
			'title': {
				pattern: /^.+(?:\r?\n|\r)(?:={3,}|-{3,}|~{3,}|\^{3,}|\+{3,})$|^={1,5} .+|^\.(?![\s.]).*/m,
				alias: 'important',
				inside: {
					'punctuation': /^(?:\.|=+)|(?:=+|-+|~+|\^+|\++)$/
					// See rest below
				}
			},
			'attribute-entry': {
				pattern: /^:[^:\r\n]+:(?: .*?(?: \+(?:\r?\n|\r).*?)*)?$/m,
				alias: 'tag'
			},
			'attributes': attributes,
			'hr': {
				pattern: /^'{3,}$/m,
				alias: 'punctuation'
			},
			'page-break': {
				pattern: /^<{3,}$/m,
				alias: 'punctuation'
			},
			'admonition': {
				pattern: /^(?:TIP|NOTE|IMPORTANT|WARNING|CAUTION):/m,
				alias: 'keyword'
			},
			'callout': [
				{
					pattern: /(^[ \t]*)<?\d*>/m,
					lookbehind: true,
					alias: 'symbol'
				},
				{
					pattern: /<\d+>/,
					alias: 'symbol'
				}
			],
			'macro': {
				pattern: /\b[a-z\d][a-z\d-]*::?(?:[^\s\[\]]*\[(?:[^\]\\"']|(["'])(?:(?!\1)[^\\]|\\.)*\1|\\.)*\])/,
				inside: {
					'function': /^[a-z\d-]+(?=:)/,
					'punctuation': /^::?/,
					'attributes': {
						pattern: /(?:\[(?:[^\]\\"']|(["'])(?:(?!\1)[^\\]|\\.)*\1|\\.)*\])/,
						inside: attributes.inside
					}
				}
			},
			'inline': {
				/*
				The initial look-behind prevents the highlighting of escaped quoted text.

				Quoted text can be multi-line but cannot span an empty line.
				All quoted text can have attributes before [foobar, 'foobar', baz="bar"].

				First, we handle the constrained quotes.
				Those must be bounded by non-word chars and cannot have spaces between the delimiter and the first char.
				They are, in order: _emphasis_, ``double quotes'', `single quotes', `monospace`, 'emphasis', *strong*, +monospace+ and #unquoted#

				Then we handle the unconstrained quotes.
				Those do not have the restrictions of the constrained quotes.
				They are, in order: __emphasis__, **strong**, ++monospace++, +++passthrough+++, ##unquoted##, $$passthrough$$, ~subscript~, ^superscript^, {attribute-reference}, [[anchor]], [[[bibliography anchor]]], <<xref>>, (((indexes))) and ((indexes))
				 */
				pattern: /(^|[^\\])(?:(?:\B\[(?:[^\]\\"']|(["'])(?:(?!\2)[^\\]|\\.)*\2|\\.)*\])?(?:\b_(?!\s)(?: _|[^_\\\r\n]|\\.)+(?:(?:\r?\n|\r)(?: _|[^_\\\r\n]|\\.)+)*_\b|\B``(?!\s).+?(?:(?:\r?\n|\r).+?)*''\B|\B`(?!\s)(?:[^`'\s]|\s+\S)+['`]\B|\B(['*+#])(?!\s)(?: \3|(?!\3)[^\\\r\n]|\\.)+(?:(?:\r?\n|\r)(?: \3|(?!\3)[^\\\r\n]|\\.)+)*\3\B)|(?:\[(?:[^\]\\"']|(["'])(?:(?!\4)[^\\]|\\.)*\4|\\.)*\])?(?:(__|\*\*|\+\+\+?|##|\$\$|[~^]).+?(?:(?:\r?\n|\r).+?)*\5|\{[^}\r\n]+\}|\[\[\[?.+?(?:(?:\r?\n|\r).+?)*\]?\]\]|<<.+?(?:(?:\r?\n|\r).+?)*>>|\(\(\(?.+?(?:(?:\r?\n|\r).+?)*\)?\)\)))/m,
				lookbehind: true,
				inside: {
					'attributes': attributes,
					'url': {
						pattern: /^(?:\[\[\[?.+?\]?\]\]|<<.+?>>)$/,
						inside: {
							'punctuation': /^(?:\[\[\[?|<<)|(?:\]\]\]?|>>)$/
						}
					},
					'attribute-ref': {
						pattern: /^\{.+\}$/,
						inside: {
							'variable': {
								pattern: /(^\{)[a-z\d,+_-]+/,
								lookbehind: true
							},
							'operator': /^[=?!#%@$]|!(?=[:}])/,
							'punctuation': /^\{|\}$|::?/
						}
					},
					'italic': {
						pattern: /^(['_])[\s\S]+\1$/,
						inside: {
							'punctuation': /^(?:''?|__?)|(?:''?|__?)$/
						}
					},
					'bold': {
						pattern: /^\*[\s\S]+\*$/,
						inside: {
							punctuation: /^\*\*?|\*\*?$/
						}
					},
					'punctuation': /^(?:``?|\+{1,3}|##?|\$\$|[~^]|\(\(\(?)|(?:''?|\+{1,3}|##?|\$\$|[~^`]|\)?\)\))$/
				}
			},
			'replacement': {
				pattern: /\((?:C|TM|R)\)/,
				alias: 'builtin'
			},
			'entity': /&#?[\da-z]{1,8};/i,
			'line-continuation': {
				pattern: /(^| )\+$/m,
				lookbehind: true,
				alias: 'punctuation'
			}
		};


		// Allow some nesting. There is no recursion though, so cloning should not be needed.

		function copyFromAsciiDoc(keys) {
			keys = keys.split(' ');

			var o = {};
			for (var i = 0, l = keys.length; i < l; i++) {
				o[keys[i]] = asciidoc[keys[i]];
			}
			return o;
		}

		attributes.inside['interpreted'].inside.rest = copyFromAsciiDoc('macro inline replacement entity');

		asciidoc['passthrough-block'].inside.rest = copyFromAsciiDoc('macro');

		asciidoc['literal-block'].inside.rest = copyFromAsciiDoc('callout');

		asciidoc['table'].inside.rest = copyFromAsciiDoc('comment-block passthrough-block literal-block other-block list-punctuation indented-block comment title attribute-entry attributes hr page-break admonition list-label callout macro inline replacement entity line-continuation');

		asciidoc['other-block'].inside.rest = copyFromAsciiDoc('table list-punctuation indented-block comment attribute-entry attributes hr page-break admonition list-label macro inline replacement entity line-continuation');

		asciidoc['title'].inside.rest = copyFromAsciiDoc('macro inline replacement entity');


		// Plugin to make entity title show the real entity, idea by Roman Komarov
		Prism.hooks.add('wrap', function (env) {
			if (env.type === 'entity') {
				env.attributes['title'] = env.content.replace(/&amp;/, '&');
			}
		});

		Prism.languages.adoc = Prism.languages.asciidoc;
	}(Prism));

	(function (Prism) {

		/**
		 * Replaces all placeholders "<<n>>" of given pattern with the n-th replacement (zero based).
		 *
		 * Note: This is a simple text based replacement. Be careful when using backreferences!
		 *
		 * @param {string} pattern the given pattern.
		 * @param {string[]} replacements a list of replacement which can be inserted into the given pattern.
		 * @returns {string} the pattern with all placeholders replaced with their corresponding replacements.
		 * @example replace(/a<<0>>a/.source, [/b+/.source]) === /a(?:b+)a/.source
		 */
		function replace(pattern, replacements) {
			return pattern.replace(/<<(\d+)>>/g, function (m, index) {
				return '(?:' + replacements[+index] + ')';
			});
		}
		/**
		 * @param {string} pattern
		 * @param {string[]} replacements
		 * @param {string} [flags]
		 * @returns {RegExp}
		 */
		function re(pattern, replacements, flags) {
			return RegExp(replace(pattern, replacements), flags || '');
		}

		/**
		 * Creates a nested pattern where all occurrences of the string `<<self>>` are replaced with the pattern itself.
		 *
		 * @param {string} pattern
		 * @param {number} depthLog2
		 * @returns {string}
		 */
		function nested(pattern, depthLog2) {
			for (var i = 0; i < depthLog2; i++) {
				pattern = pattern.replace(/<<self>>/g, function () { return '(?:' + pattern + ')'; });
			}
			return pattern.replace(/<<self>>/g, '[^\\s\\S]');
		}

		// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/
		var keywordKinds = {
			// keywords which represent a return or variable type
			type: 'bool byte char decimal double dynamic float int long object sbyte short string uint ulong ushort var void',
			// keywords which are used to declare a type
			typeDeclaration: 'class enum interface struct',
			// contextual keywords
			// ("var" and "dynamic" are missing because they are used like types)
			contextual: 'add alias and ascending async await by descending from get global group into join let nameof not notnull on or orderby partial remove select set unmanaged value when where',
			// all other keywords
			other: 'abstract as base break case catch checked const continue default delegate do else event explicit extern finally fixed for foreach goto if implicit in internal is lock namespace new null operator out override params private protected public readonly ref return sealed sizeof stackalloc static switch this throw try typeof unchecked unsafe using virtual volatile while yield'
		};

		// keywords
		function keywordsToPattern(words) {
			return '\\b(?:' + words.trim().replace(/ /g, '|') + ')\\b';
		}
		var typeDeclarationKeywords = keywordsToPattern(keywordKinds.typeDeclaration);
		var keywords = RegExp(keywordsToPattern(keywordKinds.type + ' ' + keywordKinds.typeDeclaration + ' ' + keywordKinds.contextual + ' ' + keywordKinds.other));
		var nonTypeKeywords = keywordsToPattern(keywordKinds.typeDeclaration + ' ' + keywordKinds.contextual + ' ' + keywordKinds.other);
		var nonContextualKeywords = keywordsToPattern(keywordKinds.type + ' ' + keywordKinds.typeDeclaration + ' ' + keywordKinds.other);

		// types
		var generic = nested(/<(?:[^<>;=+\-*/%&|^]|<<self>>)*>/.source, 2); // the idea behind the other forbidden characters is to prevent false positives. Same for tupleElement.
		var nestedRound = nested(/\((?:[^()]|<<self>>)*\)/.source, 2);
		var name = /@?\b[A-Za-z_]\w*\b/.source;
		var genericName = replace(/<<0>>(?:\s*<<1>>)?/.source, [name, generic]);
		var identifier = replace(/(?!<<0>>)<<1>>(?:\s*\.\s*<<1>>)*/.source, [nonTypeKeywords, genericName]);
		var array = /\[\s*(?:,\s*)*\]/.source;
		var typeExpressionWithoutTuple = replace(/<<0>>(?:\s*(?:\?\s*)?<<1>>)*(?:\s*\?)?/.source, [identifier, array]);
		var tupleElement = replace(/[^,()<>[\];=+\-*/%&|^]|<<0>>|<<1>>|<<2>>/.source, [generic, nestedRound, array]);
		var tuple = replace(/\(<<0>>+(?:,<<0>>+)+\)/.source, [tupleElement]);
		var typeExpression = replace(/(?:<<0>>|<<1>>)(?:\s*(?:\?\s*)?<<2>>)*(?:\s*\?)?/.source, [tuple, identifier, array]);

		var typeInside = {
			'keyword': keywords,
			'punctuation': /[<>()?,.:[\]]/
		};

		// strings & characters
		// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/lexical-structure#character-literals
		// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/lexical-structure#string-literals
		var character = /'(?:[^\r\n'\\]|\\.|\\[Uux][\da-fA-F]{1,8})'/.source; // simplified pattern
		var regularString = /"(?:\\.|[^\\"\r\n])*"/.source;
		var verbatimString = /@"(?:""|\\[\s\S]|[^\\"])*"(?!")/.source;


		Prism.languages.csharp = Prism.languages.extend('clike', {
			'string': [
				{
					pattern: re(/(^|[^$\\])<<0>>/.source, [verbatimString]),
					lookbehind: true,
					greedy: true
				},
				{
					pattern: re(/(^|[^@$\\])<<0>>/.source, [regularString]),
					lookbehind: true,
					greedy: true
				},
				{
					pattern: RegExp(character),
					greedy: true,
					alias: 'character'
				}
			],
			'class-name': [
				{
					// Using static
					// using static System.Math;
					pattern: re(/(\busing\s+static\s+)<<0>>(?=\s*;)/.source, [identifier]),
					lookbehind: true,
					inside: typeInside
				},
				{
					// Using alias (type)
					// using Project = PC.MyCompany.Project;
					pattern: re(/(\busing\s+<<0>>\s*=\s*)<<1>>(?=\s*;)/.source, [name, typeExpression]),
					lookbehind: true,
					inside: typeInside
				},
				{
					// Using alias (alias)
					// using Project = PC.MyCompany.Project;
					pattern: re(/(\busing\s+)<<0>>(?=\s*=)/.source, [name]),
					lookbehind: true
				},
				{
					// Type declarations
					// class Foo<A, B>
					// interface Foo<out A, B>
					pattern: re(/(\b<<0>>\s+)<<1>>/.source, [typeDeclarationKeywords, genericName]),
					lookbehind: true,
					inside: typeInside
				},
				{
					// Single catch exception declaration
					// catch(Foo)
					// (things like catch(Foo e) is covered by variable declaration)
					pattern: re(/(\bcatch\s*\(\s*)<<0>>/.source, [identifier]),
					lookbehind: true,
					inside: typeInside
				},
				{
					// Name of the type parameter of generic constraints
					// where Foo : class
					pattern: re(/(\bwhere\s+)<<0>>/.source, [name]),
					lookbehind: true
				},
				{
					// Casts and checks via as and is.
					// as Foo<A>, is Bar<B>
					// (things like if(a is Foo b) is covered by variable declaration)
					pattern: re(/(\b(?:is(?:\s+not)?|as)\s+)<<0>>/.source, [typeExpressionWithoutTuple]),
					lookbehind: true,
					inside: typeInside
				},
				{
					// Variable, field and parameter declaration
					// (Foo bar, Bar baz, Foo[,,] bay, Foo<Bar, FooBar<Bar>> bax)
					pattern: re(/\b<<0>>(?=\s+(?!<<1>>)<<2>>(?:\s*[=,;:{)\]]|\s+(?:in|when)\b))/.source, [typeExpression, nonContextualKeywords, name]),
					inside: typeInside
				}
			],
			'keyword': keywords,
			// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/lexical-structure#literals
			'number': /(?:\b0(?:x[\da-f_]*[\da-f]|b[01_]*[01])|(?:\B\.\d+(?:_+\d+)*|\b\d+(?:_+\d+)*(?:\.\d+(?:_+\d+)*)?)(?:e[-+]?\d+(?:_+\d+)*)?)(?:ul|lu|[dflmu])?\b/i,
			'operator': />>=?|<<=?|[-=]>|([-+&|])\1|~|\?\?=?|[-+*/%&|^!=<>]=?/,
			'punctuation': /\?\.?|::|[{}[\];(),.:]/
		});

		Prism.languages.insertBefore('csharp', 'number', {
			'range': {
				pattern: /\.\./,
				alias: 'operator'
			}
		});

		Prism.languages.insertBefore('csharp', 'punctuation', {
			'named-parameter': {
				pattern: re(/([(,]\s*)<<0>>(?=\s*:)/.source, [name]),
				lookbehind: true,
				alias: 'punctuation'
			}
		});

		Prism.languages.insertBefore('csharp', 'class-name', {
			'namespace': {
				// namespace Foo.Bar {}
				// using Foo.Bar;
				pattern: re(/(\b(?:namespace|using)\s+)<<0>>(?:\s*\.\s*<<0>>)*(?=\s*[;{])/.source, [name]),
				lookbehind: true,
				inside: {
					'punctuation': /\./
				}
			},
			'type-expression': {
				// default(Foo), typeof(Foo<Bar>), sizeof(int)
				pattern: re(/(\b(?:default|typeof|sizeof)\s*\(\s*(?!\s))(?:[^()\s]|\s(?!\s)|<<0>>)*(?=\s*\))/.source, [nestedRound]),
				lookbehind: true,
				alias: 'class-name',
				inside: typeInside
			},
			'return-type': {
				// Foo<Bar> ForBar(); Foo IFoo.Bar() => 0
				// int this[int index] => 0; T IReadOnlyList<T>.this[int index] => this[index];
				// int Foo => 0; int Foo { get; set } = 0;
				pattern: re(/<<0>>(?=\s+(?:<<1>>\s*(?:=>|[({]|\.\s*this\s*\[)|this\s*\[))/.source, [typeExpression, identifier]),
				inside: typeInside,
				alias: 'class-name'
			},
			'constructor-invocation': {
				// new List<Foo<Bar[]>> { }
				pattern: re(/(\bnew\s+)<<0>>(?=\s*[[({])/.source, [typeExpression]),
				lookbehind: true,
				inside: typeInside,
				alias: 'class-name'
			},
			/*'explicit-implementation': {
				// int IFoo<Foo>.Bar => 0; void IFoo<Foo<Foo>>.Foo<T>();
				pattern: replace(/\b<<0>>(?=\.<<1>>)/, className, methodOrPropertyDeclaration),
				inside: classNameInside,
				alias: 'class-name'
			},*/
			'generic-method': {
				// foo<Bar>()
				pattern: re(/<<0>>\s*<<1>>(?=\s*\()/.source, [name, generic]),
				inside: {
					'function': re(/^<<0>>/.source, [name]),
					'generic': {
						pattern: RegExp(generic),
						alias: 'class-name',
						inside: typeInside
					}
				}
			},
			'type-list': {
				// The list of types inherited or of generic constraints
				// class Foo<F> : Bar, IList<FooBar>
				// where F : Bar, IList<int>
				pattern: re(
					/\b((?:<<0>>\s+<<1>>|where\s+<<2>>)\s*:\s*)(?:<<3>>|<<4>>)(?:\s*,\s*(?:<<3>>|<<4>>))*(?=\s*(?:where|[{;]|=>|$))/.source,
					[typeDeclarationKeywords, genericName, name, typeExpression, keywords.source]
				),
				lookbehind: true,
				inside: {
					'keyword': keywords,
					'class-name': {
						pattern: RegExp(typeExpression),
						greedy: true,
						inside: typeInside
					},
					'punctuation': /,/
				}
			},
			'preprocessor': {
				pattern: /(^\s*)#.*/m,
				lookbehind: true,
				alias: 'property',
				inside: {
					// highlight preprocessor directives as keywords
					'directive': {
						pattern: /(\s*#)\b(?:define|elif|else|endif|endregion|error|if|line|pragma|region|undef|warning)\b/,
						lookbehind: true,
						alias: 'keyword'
					}
				}
			}
		});

		// attributes
		var regularStringOrCharacter = regularString + '|' + character;
		var regularStringCharacterOrComment = replace(/\/(?![*/])|\/\/[^\r\n]*[\r\n]|\/\*(?:[^*]|\*(?!\/))*\*\/|<<0>>/.source, [regularStringOrCharacter]);
		var roundExpression = nested(replace(/[^"'/()]|<<0>>|\(<<self>>*\)/.source, [regularStringCharacterOrComment]), 2);

		// https://docs.microsoft.com/en-us/dotnet/csharp/programming-guide/concepts/attributes/#attribute-targets
		var attrTarget = /\b(?:assembly|event|field|method|module|param|property|return|type)\b/.source;
		var attr = replace(/<<0>>(?:\s*\(<<1>>*\))?/.source, [identifier, roundExpression]);

		Prism.languages.insertBefore('csharp', 'class-name', {
			'attribute': {
				// Attributes
				// [Foo], [Foo(1), Bar(2, Prop = "foo")], [return: Foo(1), Bar(2)], [assembly: Foo(Bar)]
				pattern: re(/((?:^|[^\s\w>)?])\s*\[\s*)(?:<<0>>\s*:\s*)?<<1>>(?:\s*,\s*<<1>>)*(?=\s*\])/.source, [attrTarget, attr]),
				lookbehind: true,
				greedy: true,
				inside: {
					'target': {
						pattern: re(/^<<0>>(?=\s*:)/.source, [attrTarget]),
						alias: 'keyword'
					},
					'attribute-arguments': {
						pattern: re(/\(<<0>>*\)/.source, [roundExpression]),
						inside: Prism.languages.csharp
					},
					'class-name': {
						pattern: RegExp(identifier),
						inside: {
							'punctuation': /\./
						}
					},
					'punctuation': /[:,]/
				}
			}
		});


		// string interpolation
		var formatString = /:[^}\r\n]+/.source;
		// multi line
		var mInterpolationRound = nested(replace(/[^"'/()]|<<0>>|\(<<self>>*\)/.source, [regularStringCharacterOrComment]), 2);
		var mInterpolation = replace(/\{(?!\{)(?:(?![}:])<<0>>)*<<1>>?\}/.source, [mInterpolationRound, formatString]);
		// single line
		var sInterpolationRound = nested(replace(/[^"'/()]|\/(?!\*)|\/\*(?:[^*]|\*(?!\/))*\*\/|<<0>>|\(<<self>>*\)/.source, [regularStringOrCharacter]), 2);
		var sInterpolation = replace(/\{(?!\{)(?:(?![}:])<<0>>)*<<1>>?\}/.source, [sInterpolationRound, formatString]);

		function createInterpolationInside(interpolation, interpolationRound) {
			return {
				'interpolation': {
					pattern: re(/((?:^|[^{])(?:\{\{)*)<<0>>/.source, [interpolation]),
					lookbehind: true,
					inside: {
						'format-string': {
							pattern: re(/(^\{(?:(?![}:])<<0>>)*)<<1>>(?=\}$)/.source, [interpolationRound, formatString]),
							lookbehind: true,
							inside: {
								'punctuation': /^:/
							}
						},
						'punctuation': /^\{|\}$/,
						'expression': {
							pattern: /[\s\S]+/,
							alias: 'language-csharp',
							inside: Prism.languages.csharp
						}
					}
				},
				'string': /[\s\S]+/
			};
		}

		Prism.languages.insertBefore('csharp', 'string', {
			'interpolation-string': [
				{
					pattern: re(/(^|[^\\])(?:\$@|@\$)"(?:""|\\[\s\S]|\{\{|<<0>>|[^\\{"])*"/.source, [mInterpolation]),
					lookbehind: true,
					greedy: true,
					inside: createInterpolationInside(mInterpolation, mInterpolationRound),
				},
				{
					pattern: re(/(^|[^@\\])\$"(?:\\.|\{\{|<<0>>|[^\\"{])*"/.source, [sInterpolation]),
					lookbehind: true,
					greedy: true,
					inside: createInterpolationInside(sInterpolation, sInterpolationRound),
				}
			]
		});

	}(Prism));

	Prism.languages.dotnet = Prism.languages.cs = Prism.languages.csharp;

	Prism.languages.aspnet = Prism.languages.extend('markup', {
		'page-directive': {
			pattern: /<%\s*@.*%>/i,
			alias: 'tag',
			inside: {
				'page-directive': {
					pattern: /<%\s*@\s*(?:Assembly|Control|Implements|Import|Master(?:Type)?|OutputCache|Page|PreviousPageType|Reference|Register)?|%>/i,
					alias: 'tag'
				},
				rest: Prism.languages.markup.tag.inside
			}
		},
		'directive': {
			pattern: /<%.*%>/i,
			alias: 'tag',
			inside: {
				'directive': {
					pattern: /<%\s*?[$=%#:]{0,2}|%>/i,
					alias: 'tag'
				},
				rest: Prism.languages.csharp
			}
		}
	});
	// Regexp copied from prism-markup, with a negative look-ahead added
	Prism.languages.aspnet.tag.pattern = /<(?!%)\/?[^\s>\/]+(?:\s+[^\s>\/=]+(?:=(?:("|')(?:\\[\s\S]|(?!\1)[^\\])*\1|[^\s'">=]+))?)*\s*\/?>/i;

	// match directives of attribute value foo="<% Bar %>"
	Prism.languages.insertBefore('inside', 'punctuation', {
		'directive': Prism.languages.aspnet['directive']
	}, Prism.languages.aspnet.tag.inside['attr-value']);

	Prism.languages.insertBefore('aspnet', 'comment', {
		'asp-comment': {
			pattern: /<%--[\s\S]*?--%>/,
			alias: ['asp', 'comment']
		}
	});

	// script runat="server" contains csharp, not javascript
	Prism.languages.insertBefore('aspnet', Prism.languages.javascript ? 'script' : 'tag', {
		'asp-script': {
			pattern: /(<script(?=.*runat=['"]?server['"]?)[^>]*>)[\s\S]*?(?=<\/script>)/i,
			lookbehind: true,
			alias: ['asp', 'script'],
			inside: Prism.languages.csharp || {}
		}
	});

	// NOTES - follows first-first highlight method, block is locked after highlight, different from SyntaxHl
	Prism.languages.autohotkey = {
		'comment': [
			{
				pattern: /(^|\s);.*/,
				lookbehind: true
			},
			{
				pattern: /(^\s*)\/\*(?:[\r\n](?![ \t]*\*\/)|[^\r\n])*(?:[\r\n][ \t]*\*\/)?/m,
				lookbehind: true,
				greedy: true
			}
		],
		'string': /"(?:[^"\n\r]|"")*"/m,
		'tag': /^[ \t]*[^\s:]+?(?=:(?:[^:]|$))/m, //labels
		'variable': /%\w+%/,
		'number': /\b0x[\dA-Fa-f]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[Ee]-?\d+)?/,
		'operator': /\?|\/\/?=?|:=|\|[=|]?|&[=&]?|\+[=+]?|-[=-]?|\*[=*]?|<(?:<=?|>|=)?|>>?=?|[.^!=~]=?|\b(?:AND|NOT|OR)\b/,
		'boolean': /\b(?:true|false)\b/,

		'selector': /\b(?:AutoTrim|BlockInput|Break|Click|ClipWait|Continue|Control|ControlClick|ControlFocus|ControlGet|ControlGetFocus|ControlGetPos|ControlGetText|ControlMove|ControlSend|ControlSendRaw|ControlSetText|CoordMode|Critical|DetectHiddenText|DetectHiddenWindows|Drive|DriveGet|DriveSpaceFree|EnvAdd|EnvDiv|EnvGet|EnvMult|EnvSet|EnvSub|EnvUpdate|Exit|ExitApp|FileAppend|FileCopy|FileCopyDir|FileCreateDir|FileCreateShortcut|FileDelete|FileEncoding|FileGetAttrib|FileGetShortcut|FileGetSize|FileGetTime|FileGetVersion|FileInstall|FileMove|FileMoveDir|FileRead|FileReadLine|FileRecycle|FileRecycleEmpty|FileRemoveDir|FileSelectFile|FileSelectFolder|FileSetAttrib|FileSetTime|FormatTime|GetKeyState|Gosub|Goto|GroupActivate|GroupAdd|GroupClose|GroupDeactivate|Gui|GuiControl|GuiControlGet|Hotkey|ImageSearch|IniDelete|IniRead|IniWrite|Input|InputBox|KeyWait|ListHotkeys|ListLines|ListVars|Loop|Menu|MouseClick|MouseClickDrag|MouseGetPos|MouseMove|MsgBox|OnExit|OutputDebug|Pause|PixelGetColor|PixelSearch|PostMessage|Process|Progress|Random|RegDelete|RegRead|RegWrite|Reload|Repeat|Return|Run|RunAs|RunWait|Send|SendEvent|SendInput|SendMessage|SendMode|SendPlay|SendRaw|SetBatchLines|SetCapslockState|SetControlDelay|SetDefaultMouseSpeed|SetEnv|SetFormat|SetKeyDelay|SetMouseDelay|SetNumlockState|SetRegView|SetScrollLockState|SetStoreCapslockMode|SetTimer|SetTitleMatchMode|SetWinDelay|SetWorkingDir|Shutdown|Sleep|Sort|SoundBeep|SoundGet|SoundGetWaveVolume|SoundPlay|SoundSet|SoundSetWaveVolume|SplashImage|SplashTextOff|SplashTextOn|SplitPath|StatusBarGetText|StatusBarWait|StringCaseSense|StringGetPos|StringLeft|StringLen|StringLower|StringMid|StringReplace|StringRight|StringSplit|StringTrimLeft|StringTrimRight|StringUpper|Suspend|SysGet|Thread|ToolTip|Transform|TrayTip|URLDownloadToFile|WinActivate|WinActivateBottom|WinClose|WinGet|WinGetActiveStats|WinGetActiveTitle|WinGetClass|WinGetPos|WinGetText|WinGetTitle|WinHide|WinKill|WinMaximize|WinMenuSelectItem|WinMinimize|WinMinimizeAll|WinMinimizeAllUndo|WinMove|WinRestore|WinSet|WinSetTitle|WinShow|WinWait|WinWaitActive|WinWaitClose|WinWaitNotActive)\b/i,

		'constant': /\b(?:a_ahkpath|a_ahkversion|a_appdata|a_appdatacommon|a_autotrim|a_batchlines|a_caretx|a_carety|a_computername|a_controldelay|a_cursor|a_dd|a_ddd|a_dddd|a_defaultmousespeed|a_desktop|a_desktopcommon|a_detecthiddentext|a_detecthiddenwindows|a_endchar|a_eventinfo|a_exitreason|a_fileencoding|a_formatfloat|a_formatinteger|a_gui|a_guievent|a_guicontrol|a_guicontrolevent|a_guiheight|a_guiwidth|a_guix|a_guiy|a_hour|a_iconfile|a_iconhidden|a_iconnumber|a_icontip|a_index|a_ipaddress1|a_ipaddress2|a_ipaddress3|a_ipaddress4|a_is64bitos|a_isadmin|a_iscompiled|a_iscritical|a_ispaused|a_issuspended|a_isunicode|a_keydelay|a_language|a_lasterror|a_linefile|a_linenumber|a_loopfield|a_loopfileattrib|a_loopfiledir|a_loopfileext|a_loopfilefullpath|a_loopfilelongpath|a_loopfilename|a_loopfileshortname|a_loopfileshortpath|a_loopfilesize|a_loopfilesizekb|a_loopfilesizemb|a_loopfiletimeaccessed|a_loopfiletimecreated|a_loopfiletimemodified|a_loopreadline|a_loopregkey|a_loopregname|a_loopregsubkey|a_loopregtimemodified|a_loopregtype|a_mday|a_min|a_mm|a_mmm|a_mmmm|a_mon|a_mousedelay|a_msec|a_mydocuments|a_now|a_nowutc|a_numbatchlines|a_ostype|a_osversion|a_priorhotkey|a_priorkey|programfiles|a_programfiles|a_programs|a_programscommon|a_ptrsize|a_regview|a_screendpi|a_screenheight|a_screenwidth|a_scriptdir|a_scriptfullpath|a_scripthwnd|a_scriptname|a_sec|a_space|a_startmenu|a_startmenucommon|a_startup|a_startupcommon|a_stringcasesense|a_tab|a_temp|a_thisfunc|a_thishotkey|a_thislabel|a_thismenu|a_thismenuitem|a_thismenuitempos|a_tickcount|a_timeidle|a_timeidlephysical|a_timesincepriorhotkey|a_timesincethishotkey|a_titlematchmode|a_titlematchmodespeed|a_username|a_wday|a_windelay|a_windir|a_workingdir|a_yday|a_year|a_yweek|a_yyyy|clipboard|clipboardall|comspec|errorlevel)\b/i,

		'builtin': /\b(?:abs|acos|asc|asin|atan|ceil|chr|class|comobjactive|comobjarray|comobjconnect|comobjcreate|comobjerror|comobjflags|comobjget|comobjquery|comobjtype|comobjvalue|cos|dllcall|exp|fileexist|Fileopen|floor|format|il_add|il_create|il_destroy|instr|substr|isfunc|islabel|IsObject|ln|log|lv_add|lv_delete|lv_deletecol|lv_getcount|lv_getnext|lv_gettext|lv_insert|lv_insertcol|lv_modify|lv_modifycol|lv_setimagelist|ltrim|rtrim|mod|onmessage|numget|numput|registercallback|regexmatch|regexreplace|round|sin|tan|sqrt|strlen|strreplace|sb_seticon|sb_setparts|sb_settext|strsplit|tv_add|tv_delete|tv_getchild|tv_getcount|tv_getnext|tv_get|tv_getparent|tv_getprev|tv_getselection|tv_gettext|tv_modify|varsetcapacity|winactive|winexist|__New|__Call|__Get|__Set)\b/i,

		'symbol': /\b(?:alt|altdown|altup|appskey|backspace|browser_back|browser_favorites|browser_forward|browser_home|browser_refresh|browser_search|browser_stop|bs|capslock|ctrl|ctrlbreak|ctrldown|ctrlup|del|delete|down|end|enter|esc|escape|f1|f10|f11|f12|f13|f14|f15|f16|f17|f18|f19|f2|f20|f21|f22|f23|f24|f3|f4|f5|f6|f7|f8|f9|home|ins|insert|joy1|joy10|joy11|joy12|joy13|joy14|joy15|joy16|joy17|joy18|joy19|joy2|joy20|joy21|joy22|joy23|joy24|joy25|joy26|joy27|joy28|joy29|joy3|joy30|joy31|joy32|joy4|joy5|joy6|joy7|joy8|joy9|joyaxes|joybuttons|joyinfo|joyname|joypov|joyr|joyu|joyv|joyx|joyy|joyz|lalt|launch_app1|launch_app2|launch_mail|launch_media|lbutton|lcontrol|lctrl|left|lshift|lwin|lwindown|lwinup|mbutton|media_next|media_play_pause|media_prev|media_stop|numlock|numpad0|numpad1|numpad2|numpad3|numpad4|numpad5|numpad6|numpad7|numpad8|numpad9|numpadadd|numpadclear|numpaddel|numpaddiv|numpaddot|numpaddown|numpadend|numpadenter|numpadhome|numpadins|numpadleft|numpadmult|numpadpgdn|numpadpgup|numpadright|numpadsub|numpadup|pgdn|pgup|printscreen|ralt|rbutton|rcontrol|rctrl|right|rshift|rwin|rwindown|rwinup|scrolllock|shift|shiftdown|shiftup|space|tab|up|volume_down|volume_mute|volume_up|wheeldown|wheelleft|wheelright|wheelup|xbutton1|xbutton2)\b/i,

		'important': /#\b(?:AllowSameLineComments|ClipboardTimeout|CommentFlag|DerefChar|ErrorStdOut|EscapeChar|HotkeyInterval|HotkeyModifierTimeout|Hotstring|If|IfTimeout|IfWinActive|IfWinExist|IfWinNotActive|IfWinNotExist|Include|IncludeAgain|InputLevel|InstallKeybdHook|InstallMouseHook|KeyHistory|MaxHotkeysPerInterval|MaxMem|MaxThreads|MaxThreadsBuffer|MaxThreadsPerHotkey|MenuMaskKey|NoEnv|NoTrayIcon|Persistent|SingleInstance|UseHook|Warn|WinActivateForce)\b/i,

		'keyword': /\b(?:Abort|AboveNormal|Add|ahk_class|ahk_exe|ahk_group|ahk_id|ahk_pid|All|Alnum|Alpha|AltSubmit|AltTab|AltTabAndMenu|AltTabMenu|AltTabMenuDismiss|AlwaysOnTop|AutoSize|Background|BackgroundTrans|BelowNormal|between|BitAnd|BitNot|BitOr|BitShiftLeft|BitShiftRight|BitXOr|Bold|Border|Button|ByRef|Checkbox|Checked|CheckedGray|Choose|ChooseString|Close|Color|ComboBox|Contains|ControlList|Count|Date|DateTime|Days|DDL|Default|DeleteAll|Delimiter|Deref|Destroy|Digit|Disable|Disabled|DropDownList|Edit|Eject|Else|Enable|Enabled|Error|Exist|Expand|ExStyle|FileSystem|First|Flash|Float|FloatFast|Focus|Font|for|global|Grid|Group|GroupBox|GuiClose|GuiContextMenu|GuiDropFiles|GuiEscape|GuiSize|Hdr|Hidden|Hide|High|HKCC|HKCR|HKCU|HKEY_CLASSES_ROOT|HKEY_CURRENT_CONFIG|HKEY_CURRENT_USER|HKEY_LOCAL_MACHINE|HKEY_USERS|HKLM|HKU|Hours|HScroll|Icon|IconSmall|ID|IDLast|If|IfEqual|IfExist|IfGreater|IfGreaterOrEqual|IfInString|IfLess|IfLessOrEqual|IfMsgBox|IfNotEqual|IfNotExist|IfNotInString|IfWinActive|IfWinExist|IfWinNotActive|IfWinNotExist|Ignore|ImageList|in|Integer|IntegerFast|Interrupt|is|italic|Join|Label|LastFound|LastFoundExist|Limit|Lines|List|ListBox|ListView|local|Lock|Logoff|Low|Lower|Lowercase|MainWindow|Margin|Maximize|MaximizeBox|MaxSize|Minimize|MinimizeBox|MinMax|MinSize|Minutes|MonthCal|Mouse|Move|Multi|NA|No|NoActivate|NoDefault|NoHide|NoIcon|NoMainWindow|norm|Normal|NoSort|NoSortHdr|NoStandard|Not|NoTab|NoTimers|Number|Off|Ok|On|OwnDialogs|Owner|Parse|Password|Picture|Pixel|Pos|Pow|Priority|ProcessName|Radio|Range|Read|ReadOnly|Realtime|Redraw|REG_BINARY|REG_DWORD|REG_EXPAND_SZ|REG_MULTI_SZ|REG_SZ|Region|Relative|Rename|Report|Resize|Restore|Retry|RGB|Screen|Seconds|Section|Serial|SetLabel|ShiftAltTab|Show|Single|Slider|SortDesc|Standard|static|Status|StatusBar|StatusCD|strike|Style|Submit|SysMenu|Tab2|TabStop|Text|Theme|Tile|ToggleCheck|ToggleEnable|ToolWindow|Top|Topmost|TransColor|Transparent|Tray|TreeView|TryAgain|Throw|Try|Catch|Finally|Type|UnCheck|underline|Unicode|Unlock|Until|UpDown|Upper|Uppercase|UseErrorLevel|Vis|VisFirst|Visible|VScroll|Wait|WaitClose|WantCtrlA|WantF2|WantReturn|While|Wrap|Xdigit|xm|xp|xs|Yes|ym|yp|ys)\b/i,
		'function': /[^(); \t,\n+*\-=?>:\\\/<&%\[\]]+?(?=\()/m,
		'punctuation': /[{}[\]():,]/
	};

	(function (Prism) {
		// $ set | grep '^[A-Z][^[:space:]]*=' | cut -d= -f1 | tr '\n' '|'
		// + LC_ALL, RANDOM, REPLY, SECONDS.
		// + make sure PS1..4 are here as they are not always set,
		// - some useless things.
		var envVars = '\\b(?:BASH|BASHOPTS|BASH_ALIASES|BASH_ARGC|BASH_ARGV|BASH_CMDS|BASH_COMPLETION_COMPAT_DIR|BASH_LINENO|BASH_REMATCH|BASH_SOURCE|BASH_VERSINFO|BASH_VERSION|COLORTERM|COLUMNS|COMP_WORDBREAKS|DBUS_SESSION_BUS_ADDRESS|DEFAULTS_PATH|DESKTOP_SESSION|DIRSTACK|DISPLAY|EUID|GDMSESSION|GDM_LANG|GNOME_KEYRING_CONTROL|GNOME_KEYRING_PID|GPG_AGENT_INFO|GROUPS|HISTCONTROL|HISTFILE|HISTFILESIZE|HISTSIZE|HOME|HOSTNAME|HOSTTYPE|IFS|INSTANCE|JOB|LANG|LANGUAGE|LC_ADDRESS|LC_ALL|LC_IDENTIFICATION|LC_MEASUREMENT|LC_MONETARY|LC_NAME|LC_NUMERIC|LC_PAPER|LC_TELEPHONE|LC_TIME|LESSCLOSE|LESSOPEN|LINES|LOGNAME|LS_COLORS|MACHTYPE|MAILCHECK|MANDATORY_PATH|NO_AT_BRIDGE|OLDPWD|OPTERR|OPTIND|ORBIT_SOCKETDIR|OSTYPE|PAPERSIZE|PATH|PIPESTATUS|PPID|PS1|PS2|PS3|PS4|PWD|RANDOM|REPLY|SECONDS|SELINUX_INIT|SESSION|SESSIONTYPE|SESSION_MANAGER|SHELL|SHELLOPTS|SHLVL|SSH_AUTH_SOCK|TERM|UID|UPSTART_EVENTS|UPSTART_INSTANCE|UPSTART_JOB|UPSTART_SESSION|USER|WINDOWID|XAUTHORITY|XDG_CONFIG_DIRS|XDG_CURRENT_DESKTOP|XDG_DATA_DIRS|XDG_GREETER_DATA_DIR|XDG_MENU_PREFIX|XDG_RUNTIME_DIR|XDG_SEAT|XDG_SEAT_PATH|XDG_SESSION_DESKTOP|XDG_SESSION_ID|XDG_SESSION_PATH|XDG_SESSION_TYPE|XDG_VTNR|XMODIFIERS)\\b';

		var commandAfterHeredoc = {
			pattern: /(^(["']?)\w+\2)[ \t]+\S.*/,
			lookbehind: true,
			alias: 'punctuation', // this looks reasonably well in all themes
			inside: null // see below
		};

		var insideString = {
			'bash': commandAfterHeredoc,
			'environment': {
				pattern: RegExp('\\$' + envVars),
				alias: 'constant'
			},
			'variable': [
				// [0]: Arithmetic Environment
				{
					pattern: /\$?\(\([\s\S]+?\)\)/,
					greedy: true,
					inside: {
						// If there is a $ sign at the beginning highlight $(( and )) as variable
						'variable': [
							{
								pattern: /(^\$\(\([\s\S]+)\)\)/,
								lookbehind: true
							},
							/^\$\(\(/
						],
						'number': /\b0x[\dA-Fa-f]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[Ee]-?\d+)?/,
						// Operators according to https://www.gnu.org/software/bash/manual/bashref.html#Shell-Arithmetic
						'operator': /--?|-=|\+\+?|\+=|!=?|~|\*\*?|\*=|\/=?|%=?|<<=?|>>=?|<=?|>=?|==?|&&?|&=|\^=?|\|\|?|\|=|\?|:/,
						// If there is no $ sign at the beginning highlight (( and )) as punctuation
						'punctuation': /\(\(?|\)\)?|,|;/
					}
				},
				// [1]: Command Substitution
				{
					pattern: /\$\((?:\([^)]+\)|[^()])+\)|`[^`]+`/,
					greedy: true,
					inside: {
						'variable': /^\$\(|^`|\)$|`$/
					}
				},
				// [2]: Brace expansion
				{
					pattern: /\$\{[^}]+\}/,
					greedy: true,
					inside: {
						'operator': /:[-=?+]?|[!\/]|##?|%%?|\^\^?|,,?/,
						'punctuation': /[\[\]]/,
						'environment': {
							pattern: RegExp('(\\{)' + envVars),
							lookbehind: true,
							alias: 'constant'
						}
					}
				},
				/\$(?:\w+|[#?*!@$])/
			],
			// Escape sequences from echo and printf's manuals, and escaped quotes.
			'entity': /\\(?:[abceEfnrtv\\"]|O?[0-7]{1,3}|x[0-9a-fA-F]{1,2}|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})/
		};

		Prism.languages.bash = {
			'shebang': {
				pattern: /^#!\s*\/.*/,
				alias: 'important'
			},
			'comment': {
				pattern: /(^|[^"{\\$])#.*/,
				lookbehind: true
			},
			'function-name': [
				// a) function foo {
				// b) foo() {
				// c) function foo() {
				// but not “foo {”
				{
					// a) and c)
					pattern: /(\bfunction\s+)[\w-]+(?=(?:\s*\(?:\s*\))?\s*\{)/,
					lookbehind: true,
					alias: 'function'
				},
				{
					// b)
					pattern: /\b[\w-]+(?=\s*\(\s*\)\s*\{)/,
					alias: 'function'
				}
			],
			// Highlight variable names as variables in for and select beginnings.
			'for-or-select': {
				pattern: /(\b(?:for|select)\s+)\w+(?=\s+in\s)/,
				alias: 'variable',
				lookbehind: true
			},
			// Highlight variable names as variables in the left-hand part
			// of assignments (“=” and “+=”).
			'assign-left': {
				pattern: /(^|[\s;|&]|[<>]\()\w+(?=\+?=)/,
				inside: {
					'environment': {
						pattern: RegExp('(^|[\\s;|&]|[<>]\\()' + envVars),
						lookbehind: true,
						alias: 'constant'
					}
				},
				alias: 'variable',
				lookbehind: true
			},
			'string': [
				// Support for Here-documents https://en.wikipedia.org/wiki/Here_document
				{
					pattern: /((?:^|[^<])<<-?\s*)(\w+?)\s[\s\S]*?(?:\r?\n|\r)\2/,
					lookbehind: true,
					greedy: true,
					inside: insideString
				},
				// Here-document with quotes around the tag
				// → No expansion (so no “inside”).
				{
					pattern: /((?:^|[^<])<<-?\s*)(["'])(\w+)\2\s[\s\S]*?(?:\r?\n|\r)\3/,
					lookbehind: true,
					greedy: true,
					inside: {
						'bash': commandAfterHeredoc
					}
				},
				// “Normal” string
				{
					// https://www.gnu.org/software/bash/manual/html_node/Double-Quotes.html
					pattern: /(^|[^\\](?:\\\\)*)"(?:\\[\s\S]|\$\([^)]+\)|\$(?!\()|`[^`]+`|[^"\\`$])*"/,
					lookbehind: true,
					greedy: true,
					inside: insideString
				},
				{
					// https://www.gnu.org/software/bash/manual/html_node/Single-Quotes.html
					pattern: /(^|[^$\\])'[^']*'/,
					lookbehind: true,
					greedy: true
				},
				{
					// https://www.gnu.org/software/bash/manual/html_node/ANSI_002dC-Quoting.html
					pattern: /\$'(?:[^'\\]|\\[\s\S])*'/,
					greedy: true,
					inside: {
						'entity': insideString.entity
					}
				}
			],
			'environment': {
				pattern: RegExp('\\$?' + envVars),
				alias: 'constant'
			},
			'variable': insideString.variable,
			'function': {
				pattern: /(^|[\s;|&]|[<>]\()(?:add|apropos|apt|aptitude|apt-cache|apt-get|aspell|automysqlbackup|awk|basename|bash|bc|bconsole|bg|bzip2|cal|cat|cfdisk|chgrp|chkconfig|chmod|chown|chroot|cksum|clear|cmp|column|comm|composer|cp|cron|crontab|csplit|curl|cut|date|dc|dd|ddrescue|debootstrap|df|diff|diff3|dig|dir|dircolors|dirname|dirs|dmesg|du|egrep|eject|env|ethtool|expand|expect|expr|fdformat|fdisk|fg|fgrep|file|find|fmt|fold|format|free|fsck|ftp|fuser|gawk|git|gparted|grep|groupadd|groupdel|groupmod|groups|grub-mkconfig|gzip|halt|head|hg|history|host|hostname|htop|iconv|id|ifconfig|ifdown|ifup|import|install|ip|jobs|join|kill|killall|less|link|ln|locate|logname|logrotate|look|lpc|lpr|lprint|lprintd|lprintq|lprm|ls|lsof|lynx|make|man|mc|mdadm|mkconfig|mkdir|mke2fs|mkfifo|mkfs|mkisofs|mknod|mkswap|mmv|more|most|mount|mtools|mtr|mutt|mv|nano|nc|netstat|nice|nl|nohup|notify-send|npm|nslookup|op|open|parted|passwd|paste|pathchk|ping|pkill|pnpm|popd|pr|printcap|printenv|ps|pushd|pv|quota|quotacheck|quotactl|ram|rar|rcp|reboot|remsync|rename|renice|rev|rm|rmdir|rpm|rsync|scp|screen|sdiff|sed|sendmail|seq|service|sftp|sh|shellcheck|shuf|shutdown|sleep|slocate|sort|split|ssh|stat|strace|su|sudo|sum|suspend|swapon|sync|tac|tail|tar|tee|time|timeout|top|touch|tr|traceroute|tsort|tty|umount|uname|unexpand|uniq|units|unrar|unshar|unzip|update-grub|uptime|useradd|userdel|usermod|users|uudecode|uuencode|v|vdir|vi|vim|virsh|vmstat|wait|watch|wc|wget|whereis|which|who|whoami|write|xargs|xdg-open|yarn|yes|zenity|zip|zsh|zypper)(?=$|[)\s;|&])/,
				lookbehind: true
			},
			'keyword': {
				pattern: /(^|[\s;|&]|[<>]\()(?:if|then|else|elif|fi|for|while|in|case|esac|function|select|do|done|until)(?=$|[)\s;|&])/,
				lookbehind: true
			},
			// https://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html
			'builtin': {
				pattern: /(^|[\s;|&]|[<>]\()(?:\.|:|break|cd|continue|eval|exec|exit|export|getopts|hash|pwd|readonly|return|shift|test|times|trap|umask|unset|alias|bind|builtin|caller|command|declare|echo|enable|help|let|local|logout|mapfile|printf|read|readarray|source|type|typeset|ulimit|unalias|set|shopt)(?=$|[)\s;|&])/,
				lookbehind: true,
				// Alias added to make those easier to distinguish from strings.
				alias: 'class-name'
			},
			'boolean': {
				pattern: /(^|[\s;|&]|[<>]\()(?:true|false)(?=$|[)\s;|&])/,
				lookbehind: true
			},
			'file-descriptor': {
				pattern: /\B&\d\b/,
				alias: 'important'
			},
			'operator': {
				// Lots of redirections here, but not just that.
				pattern: /\d?<>|>\||\+=|==?|!=?|=~|<<[<-]?|[&\d]?>>|\d?[<>]&?|&[>&]?|\|[&|]?|<=?|>=?/,
				inside: {
					'file-descriptor': {
						pattern: /^\d/,
						alias: 'important'
					}
				}
			},
			'punctuation': /\$?\(\(?|\)\)?|\.\.|[{}[\];\\]/,
			'number': {
				pattern: /(^|\s)(?:[1-9]\d*|0)(?:[.,]\d+)?\b/,
				lookbehind: true
			}
		};

		commandAfterHeredoc.inside = Prism.languages.bash;

		/* Patterns in command substitution. */
		var toBeCopied = [
			'comment',
			'function-name',
			'for-or-select',
			'assign-left',
			'string',
			'environment',
			'function',
			'keyword',
			'builtin',
			'boolean',
			'file-descriptor',
			'operator',
			'punctuation',
			'number'
		];
		var inside = insideString.variable[1].inside;
		for (var i = 0; i < toBeCopied.length; i++) {
			inside[toBeCopied[i]] = Prism.languages.bash[toBeCopied[i]];
		}

		Prism.languages.shell = Prism.languages.bash;
	}(Prism));

	Prism.languages.basic = {
		'comment': {
			pattern: /(?:!|REM\b).+/i,
			inside: {
				'keyword': /^REM/i
			}
		},
		'string': {
			pattern: /"(?:""|[!#$%&'()*,\/:;<=>?^_ +\-.A-Z\d])*"/i,
			greedy: true
		},
		'number': /(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:E[+-]?\d+)?/i,
		'keyword': /\b(?:AS|BEEP|BLOAD|BSAVE|CALL(?: ABSOLUTE)?|CASE|CHAIN|CHDIR|CLEAR|CLOSE|CLS|COM|COMMON|CONST|DATA|DECLARE|DEF(?: FN| SEG|DBL|INT|LNG|SNG|STR)|DIM|DO|DOUBLE|ELSE|ELSEIF|END|ENVIRON|ERASE|ERROR|EXIT|FIELD|FILES|FOR|FUNCTION|GET|GOSUB|GOTO|IF|INPUT|INTEGER|IOCTL|KEY|KILL|LINE INPUT|LOCATE|LOCK|LONG|LOOP|LSET|MKDIR|NAME|NEXT|OFF|ON(?: COM| ERROR| KEY| TIMER)?|OPEN|OPTION BASE|OUT|POKE|PUT|READ|REDIM|REM|RESTORE|RESUME|RETURN|RMDIR|RSET|RUN|SHARED|SINGLE|SELECT CASE|SHELL|SLEEP|STATIC|STEP|STOP|STRING|SUB|SWAP|SYSTEM|THEN|TIMER|TO|TROFF|TRON|TYPE|UNLOCK|UNTIL|USING|VIEW PRINT|WAIT|WEND|WHILE|WRITE)(?:\$|\b)/i,
		'function': /\b(?:ABS|ACCESS|ACOS|ANGLE|AREA|ARITHMETIC|ARRAY|ASIN|ASK|AT|ATN|BASE|BEGIN|BREAK|CAUSE|CEIL|CHR|CLIP|COLLATE|COLOR|CON|COS|COSH|COT|CSC|DATE|DATUM|DEBUG|DECIMAL|DEF|DEG|DEGREES|DELETE|DET|DEVICE|DISPLAY|DOT|ELAPSED|EPS|ERASABLE|EXLINE|EXP|EXTERNAL|EXTYPE|FILETYPE|FIXED|FP|GO|GRAPH|HANDLER|IDN|IMAGE|IN|INT|INTERNAL|IP|IS|KEYED|LBOUND|LCASE|LEFT|LEN|LENGTH|LET|LINE|LINES|LOG|LOG10|LOG2|LTRIM|MARGIN|MAT|MAX|MAXNUM|MID|MIN|MISSING|MOD|NATIVE|NUL|NUMERIC|OF|OPTION|ORD|ORGANIZATION|OUTIN|OUTPUT|PI|POINT|POINTER|POINTS|POS|PRINT|PROGRAM|PROMPT|RAD|RADIANS|RANDOMIZE|RECORD|RECSIZE|RECTYPE|RELATIVE|REMAINDER|REPEAT|REST|RETRY|REWRITE|RIGHT|RND|ROUND|RTRIM|SAME|SEC|SELECT|SEQUENTIAL|SET|SETTER|SGN|SIN|SINH|SIZE|SKIP|SQR|STANDARD|STATUS|STR|STREAM|STYLE|TAB|TAN|TANH|TEMPLATE|TEXT|THERE|TIME|TIMEOUT|TRACE|TRANSFORM|TRUNCATE|UBOUND|UCASE|USE|VAL|VARIABLE|VIEWPORT|WHEN|WINDOW|WITH|ZER|ZONEWIDTH)(?:\$|\b)/i,
		'operator': /<[=>]?|>=?|[+\-*\/^=&]|\b(?:AND|EQV|IMP|NOT|OR|XOR)\b/i,
		'punctuation': /[,;:()]/
	};

	(function (Prism) {
		var variable = /%%?[~:\w]+%?|!\S+!/;
		var parameter = {
			pattern: /\/[a-z?]+(?=[ :]|$):?|-[a-z]\b|--[a-z-]+\b/im,
			alias: 'attr-name',
			inside: {
				'punctuation': /:/
			}
		};
		var string = /"(?:[\\"]"|[^"])*"(?!")/;
		var number = /(?:\b|-)\d+\b/;

		Prism.languages.batch = {
			'comment': [
				/^::.*/m,
				{
					pattern: /((?:^|[&(])[ \t]*)rem\b(?:[^^&)\r\n]|\^(?:\r\n|[\s\S]))*/im,
					lookbehind: true
				}
			],
			'label': {
				pattern: /^:.*/m,
				alias: 'property'
			},
			'command': [
				{
					// FOR command
					pattern: /((?:^|[&(])[ \t]*)for(?: \/[a-z?](?:[ :](?:"[^"]*"|[^\s"/]\S*))?)* \S+ in \([^)]+\) do/im,
					lookbehind: true,
					inside: {
						'keyword': /^for\b|\b(?:in|do)\b/i,
						'string': string,
						'parameter': parameter,
						'variable': variable,
						'number': number,
						'punctuation': /[()',]/
					}
				},
				{
					// IF command
					pattern: /((?:^|[&(])[ \t]*)if(?: \/[a-z?](?:[ :](?:"[^"]*"|[^\s"/]\S*))?)* (?:not )?(?:cmdextversion \d+|defined \w+|errorlevel \d+|exist \S+|(?:"[^"]*"|(?!")(?:(?!==)\S)+)?(?:==| (?:equ|neq|lss|leq|gtr|geq) )(?:"[^"]*"|[^\s"]\S*))/im,
					lookbehind: true,
					inside: {
						'keyword': /^if\b|\b(?:not|cmdextversion|defined|errorlevel|exist)\b/i,
						'string': string,
						'parameter': parameter,
						'variable': variable,
						'number': number,
						'operator': /\^|==|\b(?:equ|neq|lss|leq|gtr|geq)\b/i
					}
				},
				{
					// ELSE command
					pattern: /((?:^|[&()])[ \t]*)else\b/im,
					lookbehind: true,
					inside: {
						'keyword': /^else\b/i
					}
				},
				{
					// SET command
					pattern: /((?:^|[&(])[ \t]*)set(?: \/[a-z](?:[ :](?:"[^"]*"|[^\s"/]\S*))?)* (?:[^^&)\r\n]|\^(?:\r\n|[\s\S]))*/im,
					lookbehind: true,
					inside: {
						'keyword': /^set\b/i,
						'string': string,
						'parameter': parameter,
						'variable': [
							variable,
							/\w+(?=(?:[*\/%+\-&^|]|<<|>>)?=)/
						],
						'number': number,
						'operator': /[*\/%+\-&^|]=?|<<=?|>>=?|[!~_=]/,
						'punctuation': /[()',]/
					}
				},
				{
					// Other commands
					pattern: /((?:^|[&(])[ \t]*@?)\w+\b(?:"(?:[\\"]"|[^"])*"(?!")|[^"^&)\r\n]|\^(?:\r\n|[\s\S]))*/im,
					lookbehind: true,
					inside: {
						'keyword': /^\w+\b/i,
						'string': string,
						'parameter': parameter,
						'label': {
							pattern: /(^\s*):\S+/m,
							lookbehind: true,
							alias: 'property'
						},
						'variable': variable,
						'number': number,
						'operator': /\^/
					}
				}
			],
			'operator': /[&@]/,
			'punctuation': /[()']/
		};
	}(Prism));

	Prism.languages.bnf = {
		'string': {
			pattern: /"[^\r\n"]*"|'[^\r\n']*'/
		},
		'definition': {
			pattern: /<[^<>\r\n\t]+>(?=\s*::=)/,
			alias: ['rule', 'keyword'],
			inside: {
				'punctuation': /^<|>$/
			}
		},
		'rule': {
			pattern: /<[^<>\r\n\t]+>/,
			inside: {
				'punctuation': /^<|>$/
			}
		},
		'operator': /::=|[|()[\]{}*+?]|\.{3}/
	};

	Prism.languages.rbnf = Prism.languages.bnf;

	Prism.languages.brainfuck = {
		'pointer': {
			pattern: /<|>/,
			alias: 'keyword'
		},
		'increment': {
			pattern: /\+/,
			alias: 'inserted'
		},
		'decrement': {
			pattern: /-/,
			alias: 'deleted'
		},
		'branching': {
			pattern: /\[|\]/,
			alias: 'important'
		},
		'operator': /[.,]/,
		'comment': /\S+/
	};
	Prism.languages.brightscript = {
		'comment': /(?:\brem|').*/i,
		'directive-statement': {
			pattern: /(^[\t ]*)#(?:const|else(?:[\t ]+if)?|end[\t ]+if|error|if).*/im,
			lookbehind: true,
			alias: 'property',
			inside: {
				'error-message': {
					pattern: /(^#error).+/,
					lookbehind: true
				},
				'directive': {
					pattern: /^#(?:const|else(?:[\t ]+if)?|end[\t ]+if|error|if)/,
					alias: 'keyword'
				},
				'expression': {
					pattern: /[\s\S]+/,
					inside: null // see below
				}
			}
		},
		'property': {
			pattern: /([\r\n{,][\t ]*)(?:(?!\d)\w+|"(?:[^"\r\n]|"")*"(?!"))(?=[ \t]*:)/,
			lookbehind: true,
			greedy: true
		},
		'string': {
			pattern: /"(?:[^"\r\n]|"")*"(?!")/,
			greedy: true
		},
		'class-name': {
			pattern: /(\bAs[\t ]+)\w+/i,
			lookbehind: true
		},
		'keyword': /\b(?:As|Dim|Each|Else|Elseif|End|Exit|For|Function|Goto|If|In|Print|Return|Step|Stop|Sub|Then|To|While)\b/i,
		'boolean': /\b(?:true|false)\b/i,
		'function': /\b(?!\d)\w+(?=[\t ]*\()/i,
		'number': /(?:\b\d+(?:\.\d+)?(?:[ed][+-]\d+)?|&h[a-f\d]+)\b[%&!#]?/i,
		'operator': /--|\+\+|>>=?|<<=?|<>|[-+*/\\<>]=?|[:^=?]|\b(?:and|mod|not|or)\b/i,
		'punctuation': /[.,;()[\]{}]/,
		'constant': /\b(?:LINE_NUM)\b/i
	};

	Prism.languages.brightscript['directive-statement'].inside.expression.inside = Prism.languages.brightscript;

	// Copied from https://github.com/jeluard/prism-clojure
	Prism.languages.clojure = {
		'comment': /;.*/,
		'string': {
			pattern: /"(?:[^"\\]|\\.)*"/,
			greedy: true
		},
		'operator': /(?:::|[:|'])\b[a-z][\w*+!?-]*\b/i, //used for symbols and keywords
		'keyword': {
			pattern: /([^\w+*'?-])(?:def|if|do|let|\.\.|quote|var|->>|->|fn|loop|recur|throw|try|monitor-enter|\.|new|set!|def\-|defn|defn\-|defmacro|defmulti|defmethod|defstruct|defonce|declare|definline|definterface|defprotocol|==|defrecord|>=|deftype|<=|defproject|ns|\*|\+|\-|\/|<|=|>|accessor|agent|agent-errors|aget|alength|all-ns|alter|and|append-child|apply|array-map|aset|aset-boolean|aset-byte|aset-char|aset-double|aset-float|aset-int|aset-long|aset-short|assert|assoc|await|await-for|bean|binding|bit-and|bit-not|bit-or|bit-shift-left|bit-shift-right|bit-xor|boolean|branch\?|butlast|byte|cast|char|children|class|clear-agent-errors|comment|commute|comp|comparator|complement|concat|conj|cons|constantly|cond|if-not|construct-proxy|contains\?|count|create-ns|create-struct|cycle|dec|deref|difference|disj|dissoc|distinct|doall|doc|dorun|doseq|dosync|dotimes|doto|double|down|drop|drop-while|edit|end\?|ensure|eval|every\?|false\?|ffirst|file-seq|filter|find|find-doc|find-ns|find-var|first|float|flush|for|fnseq|frest|gensym|get-proxy-class|get|hash-map|hash-set|identical\?|identity|if-let|import|in-ns|inc|index|insert-child|insert-left|insert-right|inspect-table|inspect-tree|instance\?|int|interleave|intersection|into|into-array|iterate|join|key|keys|keyword|keyword\?|last|lazy-cat|lazy-cons|left|lefts|line-seq|list\*|list|load|load-file|locking|long|macroexpand|macroexpand-1|make-array|make-node|map|map-invert|map\?|mapcat|max|max-key|memfn|merge|merge-with|meta|min|min-key|name|namespace|neg\?|newline|next|nil\?|node|not|not-any\?|not-every\?|not=|ns-imports|ns-interns|ns-map|ns-name|ns-publics|ns-refers|ns-resolve|ns-unmap|nth|nthrest|or|parse|partial|path|peek|pop|pos\?|pr|pr-str|print|print-str|println|println-str|prn|prn-str|project|proxy|proxy-mappings|quot|rand|rand-int|range|re-find|re-groups|re-matcher|re-matches|re-pattern|re-seq|read|read-line|reduce|ref|ref-set|refer|rem|remove|remove-method|remove-ns|rename|rename-keys|repeat|replace|replicate|resolve|rest|resultset-seq|reverse|rfirst|right|rights|root|rrest|rseq|second|select|select-keys|send|send-off|seq|seq-zip|seq\?|set|short|slurp|some|sort|sort-by|sorted-map|sorted-map-by|sorted-set|special-symbol\?|split-at|split-with|str|string\?|struct|struct-map|subs|subvec|symbol|symbol\?|sync|take|take-nth|take-while|test|time|to-array|to-array-2d|tree-seq|true\?|union|up|update-proxy|val|vals|var-get|var-set|var\?|vector|vector-zip|vector\?|when|when-first|when-let|when-not|with-local-vars|with-meta|with-open|with-out-str|xml-seq|xml-zip|zero\?|zipmap|zipper)(?=[^\w+*'?-])/,
			lookbehind: true
		},
		'boolean': /\b(?:true|false|nil)\b/,
		'number': /\b[\da-f]+\b/i,
		'punctuation': /[{}\[\](),]/
	};

	Prism.languages.cobol = {
		'comment': {
			pattern: /\*>.*|(^[ \t]*)\*.*/m,
			lookbehind: true,
			greedy: true
		},
		'string': {
			pattern: /[xzgn]?(?:"(?:[^\r\n"]|"")*"(?!")|'(?:[^\r\n']|'')*'(?!'))/i,
			greedy: true
		},

		'level': {
			pattern: /(^[ \t]*)\d+\b/m,
			lookbehind: true,
			greedy: true,
			alias: 'number'
		},

		'class-name': {
			// https://github.com/antlr/grammars-v4/blob/42edd5b687d183b5fa679e858a82297bd27141e7/cobol85/Cobol85.g4#L1015
			pattern: /(\bpic(?:ture)?\s+)(?:(?:[-\w$/,:*+<>]|\.(?!\s|$))(?:\(\d+\))?)+/i,
			lookbehind: true,
			inside: {
				'number': {
					pattern: /(\()\d+/,
					lookbehind: true
				},
				'punctuation': /[()]/
			}
		},

		'keyword': {
			pattern: /(^|[^\w-])(?:ABORT|ACCEPT|ACCESS|ADD|ADDRESS|ADVANCING|AFTER|ALIGNED|ALL|ALPHABET|ALPHABETIC|ALPHABETIC-LOWER|ALPHABETIC-UPPER|ALPHANUMERIC|ALPHANUMERIC-EDITED|ALSO|ALTER|ALTERNATE|ANY|ARE|AREA|AREAS|AS|ASCENDING|ASCII|ASSIGN|ASSOCIATED-DATA|ASSOCIATED-DATA-LENGTH|AT|ATTRIBUTE|AUTHOR|AUTO|AUTO-SKIP|BACKGROUND-COLOR|BACKGROUND-COLOUR|BASIS|BEEP|BEFORE|BEGINNING|BELL|BINARY|BIT|BLANK|BLINK|BLOCK|BOUNDS|BOTTOM|BY|BYFUNCTION|BYTITLE|CALL|CANCEL|CAPABLE|CCSVERSION|CD|CF|CH|CHAINING|CHANGED|CHANNEL|CHARACTER|CHARACTERS|CLASS|CLASS-ID|CLOCK-UNITS|CLOSE|CLOSE-DISPOSITION|COBOL|CODE|CODE-SET|COLLATING|COL|COLUMN|COM-REG|COMMA|COMMITMENT|COMMON|COMMUNICATION|COMP|COMP-1|COMP-2|COMP-3|COMP-4|COMP-5|COMPUTATIONAL|COMPUTATIONAL-1|COMPUTATIONAL-2|COMPUTATIONAL-3|COMPUTATIONAL-4|COMPUTATIONAL-5|COMPUTE|CONFIGURATION|CONTAINS|CONTENT|CONTINUE|CONTROL|CONTROL-POINT|CONTROLS|CONVENTION|CONVERTING|COPY|CORR|CORRESPONDING|COUNT|CRUNCH|CURRENCY|CURSOR|DATA|DATA-BASE|DATE|DATE-COMPILED|DATE-WRITTEN|DAY|DAY-OF-WEEK|DBCS|DE|DEBUG-CONTENTS|DEBUG-ITEM|DEBUG-LINE|DEBUG-NAME|DEBUG-SUB-1|DEBUG-SUB-2|DEBUG-SUB-3|DEBUGGING|DECIMAL-POINT|DECLARATIVES|DEFAULT|DEFAULT-DISPLAY|DEFINITION|DELETE|DELIMITED|DELIMITER|DEPENDING|DESCENDING|DESTINATION|DETAIL|DFHRESP|DFHVALUE|DISABLE|DISK|DISPLAY|DISPLAY-1|DIVIDE|DIVISION|DONTCARE|DOUBLE|DOWN|DUPLICATES|DYNAMIC|EBCDIC|EGCS|EGI|ELSE|EMI|EMPTY-CHECK|ENABLE|END|END-ACCEPT|END-ADD|END-CALL|END-COMPUTE|END-DELETE|END-DIVIDE|END-EVALUATE|END-IF|END-MULTIPLY|END-OF-PAGE|END-PERFORM|END-READ|END-RECEIVE|END-RETURN|END-REWRITE|END-SEARCH|END-START|END-STRING|END-SUBTRACT|END-UNSTRING|END-WRITE|ENDING|ENTER|ENTRY|ENTRY-PROCEDURE|ENVIRONMENT|EOP|ERASE|ERROR|EOL|EOS|ESCAPE|ESI|EVALUATE|EVENT|EVERY|EXCEPTION|EXCLUSIVE|EXHIBIT|EXIT|EXPORT|EXTEND|EXTENDED|EXTERNAL|FD|FILE|FILE-CONTROL|FILLER|FINAL|FIRST|FOOTING|FOR|FOREGROUND-COLOR|FOREGROUND-COLOUR|FROM|FULL|FUNCTION|FUNCTIONNAME|FUNCTION-POINTER|GENERATE|GOBACK|GIVING|GLOBAL|GO|GRID|GROUP|HEADING|HIGHLIGHT|HIGH-VALUE|HIGH-VALUES|I-O|I-O-CONTROL|ID|IDENTIFICATION|IF|IMPLICIT|IMPORT|IN|INDEX|INDEXED|INDICATE|INITIAL|INITIALIZE|INITIATE|INPUT|INPUT-OUTPUT|INSPECT|INSTALLATION|INTEGER|INTO|INVALID|INVOKE|IS|JUST|JUSTIFIED|KANJI|KEPT|KEY|KEYBOARD|LABEL|LANGUAGE|LAST|LB|LD|LEADING|LEFT|LEFTLINE|LENGTH|LENGTH-CHECK|LIBACCESS|LIBPARAMETER|LIBRARY|LIMIT|LIMITS|LINAGE|LINAGE-COUNTER|LINE|LINES|LINE-COUNTER|LINKAGE|LIST|LOCAL|LOCAL-STORAGE|LOCK|LONG-DATE|LONG-TIME|LOWER|LOWLIGHT|LOW-VALUE|LOW-VALUES|MEMORY|MERGE|MESSAGE|MMDDYYYY|MODE|MODULES|MORE-LABELS|MOVE|MULTIPLE|MULTIPLY|NAMED|NATIONAL|NATIONAL-EDITED|NATIVE|NEGATIVE|NETWORK|NEXT|NO|NO-ECHO|NULL|NULLS|NUMBER|NUMERIC|NUMERIC-DATE|NUMERIC-EDITED|NUMERIC-TIME|OBJECT-COMPUTER|OCCURS|ODT|OF|OFF|OMITTED|ON|OPEN|OPTIONAL|ORDER|ORDERLY|ORGANIZATION|OTHER|OUTPUT|OVERFLOW|OVERLINE|OWN|PACKED-DECIMAL|PADDING|PAGE|PAGE-COUNTER|PASSWORD|PERFORM|PF|PH|PIC|PICTURE|PLUS|POINTER|POSITION|POSITIVE|PORT|PRINTER|PRINTING|PRIVATE|PROCEDURE|PROCEDURE-POINTER|PROCEDURES|PROCEED|PROCESS|PROGRAM|PROGRAM-ID|PROGRAM-LIBRARY|PROMPT|PURGE|QUEUE|QUOTE|QUOTES|RANDOM|READER|REMOTE|RD|REAL|READ|RECEIVE|RECEIVED|RECORD|RECORDING|RECORDS|RECURSIVE|REDEFINES|REEL|REF|REFERENCE|REFERENCES|RELATIVE|RELEASE|REMAINDER|REMARKS|REMOVAL|REMOVE|RENAMES|REPLACE|REPLACING|REPORT|REPORTING|REPORTS|REQUIRED|RERUN|RESERVE|REVERSE-VIDEO|RESET|RETURN|RETURN-CODE|RETURNING|REVERSED|REWIND|REWRITE|RF|RH|RIGHT|ROUNDED|RUN|SAME|SAVE|SCREEN|SD|SEARCH|SECTION|SECURE|SECURITY|SEGMENT|SEGMENT-LIMIT|SELECT|SEND|SENTENCE|SEPARATE|SEQUENCE|SEQUENTIAL|SET|SHARED|SHAREDBYALL|SHAREDBYRUNUNIT|SHARING|SHIFT-IN|SHIFT-OUT|SHORT-DATE|SIGN|SIZE|SORT|SORT-CONTROL|SORT-CORE-SIZE|SORT-FILE-SIZE|SORT-MERGE|SORT-MESSAGE|SORT-MODE-SIZE|SORT-RETURN|SOURCE|SOURCE-COMPUTER|SPACE|SPACES|SPECIAL-NAMES|STANDARD|STANDARD-1|STANDARD-2|START|STATUS|STOP|STRING|SUB-QUEUE-1|SUB-QUEUE-2|SUB-QUEUE-3|SUBTRACT|SUM|SUPPRESS|SYMBOL|SYMBOLIC|SYNC|SYNCHRONIZED|TABLE|TALLY|TALLYING|TASK|TAPE|TERMINAL|TERMINATE|TEST|TEXT|THEN|THREAD|THREAD-LOCAL|THROUGH|THRU|TIME|TIMER|TIMES|TITLE|TO|TODAYS-DATE|TODAYS-NAME|TOP|TRAILING|TRUNCATED|TYPE|TYPEDEF|UNDERLINE|UNIT|UNSTRING|UNTIL|UP|UPON|USAGE|USE|USING|VALUE|VALUES|VARYING|VIRTUAL|WAIT|WHEN|WHEN-COMPILED|WITH|WORDS|WORKING-STORAGE|WRITE|YEAR|YYYYMMDD|YYYYDDD|ZERO-FILL|ZEROS|ZEROES)(?![\w-])/i,
			lookbehind: true
		},

		'boolean': {
			pattern: /(^|[^\w-])(?:false|true)(?![\w-])/i,
			lookbehind: true
		},
		'number': {
			pattern: /(^|[^\w-])(?:[+-]?(?:(?:\d+(?:[.,]\d+)?|[.,]\d+)(?:e[+-]?\d+)?|zero))(?![\w-])/i,
			lookbehind: true
		},
		'operator': [
			/<>|[<>]=?|[=+*/&]/,
			{
				pattern: /(^|[^\w-])(?:-|and|equal|greater|less|not|or|than)(?![\w-])/i,
				lookbehind: true
			}
		],
		'punctuation': /[.:,()]/
	};

	(function (Prism) {

	// Ignore comments starting with { to privilege string interpolation highlighting
	var comment = /#(?!\{).+/;
	var interpolation = {
		pattern: /#\{[^}]+\}/,
		alias: 'variable'
	};

	Prism.languages.coffeescript = Prism.languages.extend('javascript', {
		'comment': comment,
		'string': [

			// Strings are multiline
			{
				pattern: /'(?:\\[\s\S]|[^\\'])*'/,
				greedy: true
			},

			{
				// Strings are multiline
				pattern: /"(?:\\[\s\S]|[^\\"])*"/,
				greedy: true,
				inside: {
					'interpolation': interpolation
				}
			}
		],
		'keyword': /\b(?:and|break|by|catch|class|continue|debugger|delete|do|each|else|extend|extends|false|finally|for|if|in|instanceof|is|isnt|let|loop|namespace|new|no|not|null|of|off|on|or|own|return|super|switch|then|this|throw|true|try|typeof|undefined|unless|until|when|while|window|with|yes|yield)\b/,
		'class-member': {
			pattern: /@(?!\d)\w+/,
			alias: 'variable'
		}
	});

	Prism.languages.insertBefore('coffeescript', 'comment', {
		'multiline-comment': {
			pattern: /###[\s\S]+?###/,
			alias: 'comment'
		},

		// Block regexp can contain comments and interpolation
		'block-regex': {
			pattern: /\/{3}[\s\S]*?\/{3}/,
			alias: 'regex',
			inside: {
				'comment': comment,
				'interpolation': interpolation
			}
		}
	});

	Prism.languages.insertBefore('coffeescript', 'string', {
		'inline-javascript': {
			pattern: /`(?:\\[\s\S]|[^\\`])*`/,
			inside: {
				'delimiter': {
					pattern: /^`|`$/,
					alias: 'punctuation'
				},
				'script': {
					pattern: /[\s\S]+/,
					alias: 'language-javascript',
					inside: Prism.languages.javascript
				}
			}
		},

		// Block strings
		'multiline-string': [
			{
				pattern: /'''[\s\S]*?'''/,
				greedy: true,
				alias: 'string'
			},
			{
				pattern: /"""[\s\S]*?"""/,
				greedy: true,
				alias: 'string',
				inside: {
					interpolation: interpolation
				}
			}
		]

	});

	Prism.languages.insertBefore('coffeescript', 'keyword', {
		// Object property
		'property': /(?!\d)\w+(?=\s*:(?!:))/
	});

	delete Prism.languages.coffeescript['template-string'];

	Prism.languages.coffee = Prism.languages.coffeescript;
	}(Prism));

	/**
	 * Original by Samuel Flores
	 *
	 * Adds the following new token classes:
	 *     constant, builtin, variable, symbol, regex
	 */
	(function (Prism) {
		Prism.languages.ruby = Prism.languages.extend('clike', {
			'comment': [
				/#.*/,
				{
					pattern: /^=begin\s[\s\S]*?^=end/m,
					greedy: true
				}
			],
			'class-name': {
				pattern: /(\b(?:class)\s+|\bcatch\s+\()[\w.\\]+/i,
				lookbehind: true,
				inside: {
					'punctuation': /[.\\]/
				}
			},
			'keyword': /\b(?:alias|and|BEGIN|begin|break|case|class|def|define_method|defined|do|each|else|elsif|END|end|ensure|extend|for|if|in|include|module|new|next|nil|not|or|prepend|protected|private|public|raise|redo|require|rescue|retry|return|self|super|then|throw|undef|unless|until|when|while|yield)\b/
		});

		var interpolation = {
			pattern: /#\{[^}]+\}/,
			inside: {
				'delimiter': {
					pattern: /^#\{|\}$/,
					alias: 'tag'
				},
				rest: Prism.languages.ruby
			}
		};

		delete Prism.languages.ruby.function;

		Prism.languages.insertBefore('ruby', 'keyword', {
			'regex': [
				{
					pattern: RegExp(/%r/.source + '(?:' + [
						/([^a-zA-Z0-9\s{(\[<])(?:(?!\1)[^\\]|\\[\s\S])*\1/.source,
						/\((?:[^()\\]|\\[\s\S])*\)/.source,
						// Here we need to specifically allow interpolation
						/\{(?:[^#{}\\]|#(?:\{[^}]+\})?|\\[\s\S])*\}/.source,
						/\[(?:[^\[\]\\]|\\[\s\S])*\]/.source,
						/<(?:[^<>\\]|\\[\s\S])*>/.source
					].join('|') + ')' + /[egimnosux]{0,6}/.source),
					greedy: true,
					inside: {
						'interpolation': interpolation
					}
				},
				{
					pattern: /(^|[^/])\/(?!\/)(?:\[[^\r\n\]]+\]|\\.|[^[/\\\r\n])+\/[egimnosux]{0,6}(?=\s*(?:$|[\r\n,.;})#]))/,
					lookbehind: true,
					greedy: true,
					inside: {
						'interpolation': interpolation
					}
				}
			],
			'variable': /[@$]+[a-zA-Z_]\w*(?:[?!]|\b)/,
			'symbol': {
				pattern: /(^|[^:]):[a-zA-Z_]\w*(?:[?!]|\b)/,
				lookbehind: true
			},
			'method-definition': {
				pattern: /(\bdef\s+)[\w.]+/,
				lookbehind: true,
				inside: {
					'function': /\w+$/,
					rest: Prism.languages.ruby
				}
			}
		});

		Prism.languages.insertBefore('ruby', 'number', {
			'builtin': /\b(?:Array|Bignum|Binding|Class|Continuation|Dir|Exception|FalseClass|File|Stat|Fixnum|Float|Hash|Integer|IO|MatchData|Method|Module|NilClass|Numeric|Object|Proc|Range|Regexp|String|Struct|TMS|Symbol|ThreadGroup|Thread|Time|TrueClass)\b/,
			'constant': /\b[A-Z]\w*(?:[?!]|\b)/
		});

		Prism.languages.ruby.string = [
			{
				pattern: RegExp(/%[qQiIwWxs]?/.source + '(?:' + [
					/([^a-zA-Z0-9\s{(\[<])(?:(?!\1)[^\\]|\\[\s\S])*\1/.source,
					/\((?:[^()\\]|\\[\s\S])*\)/.source,
					// Here we need to specifically allow interpolation
					/\{(?:[^#{}\\]|#(?:\{[^}]+\})?|\\[\s\S])*\}/.source,
					/\[(?:[^\[\]\\]|\\[\s\S])*\]/.source,
					/<(?:[^<>\\]|\\[\s\S])*>/.source
				].join('|') + ')'),
				greedy: true,
				inside: {
					'interpolation': interpolation
				}
			},
			{
				pattern: /("|')(?:#\{[^}]+\}|#(?!\{)|\\(?:\r\n|[\s\S])|(?!\1)[^\\#\r\n])*\1/,
				greedy: true,
				inside: {
					'interpolation': interpolation
				}
			}
		];

		Prism.languages.rb = Prism.languages.ruby;
	}(Prism));

	(function (Prism) {
		Prism.languages.crystal = Prism.languages.extend('ruby', {
			keyword: [
				/\b(?:abstract|alias|as|asm|begin|break|case|class|def|do|else|elsif|end|ensure|enum|extend|for|fun|if|include|instance_sizeof|lib|macro|module|next|of|out|pointerof|private|protected|rescue|return|require|select|self|sizeof|struct|super|then|type|typeof|uninitialized|union|unless|until|when|while|with|yield|__DIR__|__END_LINE__|__FILE__|__LINE__)\b/,
				{
					pattern: /(\.\s*)(?:is_a|responds_to)\?/,
					lookbehind: true
				}
			],

			number: /\b(?:0b[01_]*[01]|0o[0-7_]*[0-7]|0x[\da-fA-F_]*[\da-fA-F]|(?:\d(?:[\d_]*\d)?)(?:\.[\d_]*\d)?(?:[eE][+-]?[\d_]*\d)?)(?:_(?:[uif](?:8|16|32|64))?)?\b/
		});

		Prism.languages.insertBefore('crystal', 'string', {
			attribute: {
				pattern: /@\[.+?\]/,
				alias: 'attr-name',
				inside: {
					delimiter: {
						pattern: /^@\[|\]$/,
						alias: 'tag'
					},
					rest: Prism.languages.crystal
				}
			},

			expansion: [
			{
				pattern: /\{\{.+?\}\}/,
				inside: {
					delimiter: {
						pattern: /^\{\{|\}\}$/,
						alias: 'tag'
					},
					rest: Prism.languages.crystal
				}
			},
			{
				pattern: /\{%.+?%\}/,
				inside: {
					delimiter: {
						pattern: /^\{%|%\}$/,
						alias: 'tag'
					},
					rest: Prism.languages.crystal
				}
			}
			]
		});

	}(Prism));

	(function (Prism) {

		var string = /("|')(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/;
		var selectorInside;

		Prism.languages.css.selector = {
			pattern: Prism.languages.css.selector,
			inside: selectorInside = {
				'pseudo-element': /:(?:after|before|first-letter|first-line|selection)|::[-\w]+/,
				'pseudo-class': /:[-\w]+/,
				'class': /\.[-\w]+/,
				'id': /#[-\w]+/,
				'attribute': {
					pattern: RegExp('\\[(?:[^[\\]"\']|' + string.source + ')*\\]'),
					greedy: true,
					inside: {
						'punctuation': /^\[|\]$/,
						'case-sensitivity': {
							pattern: /(\s)[si]$/i,
							lookbehind: true,
							alias: 'keyword'
						},
						'namespace': {
							pattern: /^(\s*)(?:(?!\s)[-*\w\xA0-\uFFFF])*\|(?!=)/,
							lookbehind: true,
							inside: {
								'punctuation': /\|$/
							}
						},
						'attr-name': {
							pattern: /^(\s*)(?:(?!\s)[-\w\xA0-\uFFFF])+/,
							lookbehind: true
						},
						'attr-value': [
							string,
							{
								pattern: /(=\s*)(?:(?!\s)[-\w\xA0-\uFFFF])+(?=\s*$)/,
								lookbehind: true
							}
						],
						'operator': /[|~*^$]?=/
					}
				},
				'n-th': [
					{
						pattern: /(\(\s*)[+-]?\d*[\dn](?:\s*[+-]\s*\d+)?(?=\s*\))/,
						lookbehind: true,
						inside: {
							'number': /[\dn]+/,
							'operator': /[+-]/
						}
					},
					{
						pattern: /(\(\s*)(?:even|odd)(?=\s*\))/i,
						lookbehind: true
					}
				],
				'combinator': />|\+|~|\|\|/,

				// the `tag` token has been existed and removed.
				// because we can't find a perfect tokenize to match it.
				// if you want to add it, please read https://github.com/PrismJS/prism/pull/2373 first.

				'punctuation': /[(),]/,
			}
		};

		Prism.languages.css['atrule'].inside['selector-function-argument'].inside = selectorInside;

		Prism.languages.insertBefore('css', 'property', {
			'variable': {
				pattern: /(^|[^-\w\xA0-\uFFFF])--(?!\s)[-_a-z\xA0-\uFFFF](?:(?!\s)[-\w\xA0-\uFFFF])*/i,
				lookbehind: true
			}
		});

		var unit = {
			pattern: /(\b\d+)(?:%|[a-z]+\b)/,
			lookbehind: true
		};
		// 123 -123 .123 -.123 12.3 -12.3
		var number = {
			pattern: /(^|[^\w.-])-?(?:\d+(?:\.\d+)?|\.\d+)/,
			lookbehind: true
		};

		Prism.languages.insertBefore('css', 'function', {
			'operator': {
				pattern: /(\s)[+\-*\/](?=\s)/,
				lookbehind: true
			},
			// CAREFUL!
			// Previewers and Inline color use hexcode and color.
			'hexcode': {
				pattern: /\B#(?:[\da-f]{1,2}){3,4}\b/i,
				alias: 'color'
			},
			'color': [
				/\b(?:AliceBlue|AntiqueWhite|Aqua|Aquamarine|Azure|Beige|Bisque|Black|BlanchedAlmond|Blue|BlueViolet|Brown|BurlyWood|CadetBlue|Chartreuse|Chocolate|Coral|CornflowerBlue|Cornsilk|Crimson|Cyan|DarkBlue|DarkCyan|DarkGoldenRod|DarkGr[ae]y|DarkGreen|DarkKhaki|DarkMagenta|DarkOliveGreen|DarkOrange|DarkOrchid|DarkRed|DarkSalmon|DarkSeaGreen|DarkSlateBlue|DarkSlateGr[ae]y|DarkTurquoise|DarkViolet|DeepPink|DeepSkyBlue|DimGr[ae]y|DodgerBlue|FireBrick|FloralWhite|ForestGreen|Fuchsia|Gainsboro|GhostWhite|Gold|GoldenRod|Gr[ae]y|Green|GreenYellow|HoneyDew|HotPink|IndianRed|Indigo|Ivory|Khaki|Lavender|LavenderBlush|LawnGreen|LemonChiffon|LightBlue|LightCoral|LightCyan|LightGoldenRodYellow|LightGr[ae]y|LightGreen|LightPink|LightSalmon|LightSeaGreen|LightSkyBlue|LightSlateGr[ae]y|LightSteelBlue|LightYellow|Lime|LimeGreen|Linen|Magenta|Maroon|MediumAquaMarine|MediumBlue|MediumOrchid|MediumPurple|MediumSeaGreen|MediumSlateBlue|MediumSpringGreen|MediumTurquoise|MediumVioletRed|MidnightBlue|MintCream|MistyRose|Moccasin|NavajoWhite|Navy|OldLace|Olive|OliveDrab|Orange|OrangeRed|Orchid|PaleGoldenRod|PaleGreen|PaleTurquoise|PaleVioletRed|PapayaWhip|PeachPuff|Peru|Pink|Plum|PowderBlue|Purple|Red|RosyBrown|RoyalBlue|SaddleBrown|Salmon|SandyBrown|SeaGreen|SeaShell|Sienna|Silver|SkyBlue|SlateBlue|SlateGr[ae]y|Snow|SpringGreen|SteelBlue|Tan|Teal|Thistle|Tomato|Transparent|Turquoise|Violet|Wheat|White|WhiteSmoke|Yellow|YellowGreen)\b/i,
				{
					pattern: /\b(?:rgb|hsl)\(\s*\d{1,3}\s*,\s*\d{1,3}%?\s*,\s*\d{1,3}%?\s*\)\B|\b(?:rgb|hsl)a\(\s*\d{1,3}\s*,\s*\d{1,3}%?\s*,\s*\d{1,3}%?\s*,\s*(?:0|0?\.\d+|1)\s*\)\B/i,
					inside: {
						'unit': unit,
						'number': number,
						'function': /[\w-]+(?=\()/,
						'punctuation': /[(),]/
					}
				}
			],
			// it's important that there is no boundary assertion after the hex digits
			'entity': /\\[\da-f]{1,8}/i,
			'unit': unit,
			'number': number
		});

	}(Prism));

	// https://tools.ietf.org/html/rfc4180

	Prism.languages.csv = {
		'value': /[^\r\n,"]+|"(?:[^"]|"")*"(?!")/,
		'punctuation': /,/
	};

	Prism.languages.d = Prism.languages.extend('clike', {
		'comment': [
			{
				// Shebang
				pattern: /^\s*#!.+/,
				greedy: true
			},
			{
				pattern: RegExp(/(^|[^\\])/.source + '(?:' + [
					// /+ comment +/
					// Allow one level of nesting
					/\/\+(?:\/\+(?:[^+]|\+(?!\/))*\+\/|(?!\/\+)[\s\S])*?\+\//.source,
					// // comment
					/\/\/.*/.source,
					// /* comment */
					/\/\*[\s\S]*?\*\//.source
				].join('|') + ')'),
				lookbehind: true,
				greedy: true
			}
		],
		'string': [
			{
				pattern: RegExp([
					// r"", x""
					/\b[rx]"(?:\\[\s\S]|[^\\"])*"[cwd]?/.source,

					// q"[]", q"()", q"<>", q"{}"
					/\bq"(?:\[[\s\S]*?\]|\([\s\S]*?\)|<[\s\S]*?>|\{[\s\S]*?\})"/.source,

					// q"IDENT
					// ...
					// IDENT"
					/\bq"((?!\d)\w+)$[\s\S]*?^\1"/.source,

					// q"//", q"||", etc.
					/\bq"(.)[\s\S]*?\2"/.source,

					// Characters
					// 'a', '\\', '\n', '\xFF', '\377', '\uFFFF', '\U0010FFFF', '\quot'
					/'(?:\\(?:\W|\w+)|[^\\])'/.source,

					/(["`])(?:\\[\s\S]|(?!\3)[^\\])*\3[cwd]?/.source
				].join('|'), 'm'),
				greedy: true
			},
			{
				pattern: /\bq\{(?:\{[^{}]*\}|[^{}])*\}/,
				greedy: true,
				alias: 'token-string'
			}
		],

		// In order: $, keywords and special tokens, globally defined symbols
		'keyword': /\$|\b(?:abstract|alias|align|asm|assert|auto|body|bool|break|byte|case|cast|catch|cdouble|cent|cfloat|char|class|const|continue|creal|dchar|debug|default|delegate|delete|deprecated|do|double|else|enum|export|extern|false|final|finally|float|for|foreach|foreach_reverse|function|goto|idouble|if|ifloat|immutable|import|inout|int|interface|invariant|ireal|lazy|long|macro|mixin|module|new|nothrow|null|out|override|package|pragma|private|protected|public|pure|real|ref|return|scope|shared|short|static|struct|super|switch|synchronized|template|this|throw|true|try|typedef|typeid|typeof|ubyte|ucent|uint|ulong|union|unittest|ushort|version|void|volatile|wchar|while|with|__(?:(?:FILE|MODULE|LINE|FUNCTION|PRETTY_FUNCTION|DATE|EOF|TIME|TIMESTAMP|VENDOR|VERSION)__|gshared|traits|vector|parameters)|string|wstring|dstring|size_t|ptrdiff_t)\b/,

		'number': [
			// The lookbehind and the negative look-ahead try to prevent bad highlighting of the .. operator
			// Hexadecimal numbers must be handled separately to avoid problems with exponent "e"
			/\b0x\.?[a-f\d_]+(?:(?!\.\.)\.[a-f\d_]*)?(?:p[+-]?[a-f\d_]+)?[ulfi]{0,4}/i,
			{
				pattern: /((?:\.\.)?)(?:\b0b\.?|\b|\.)\d[\d_]*(?:(?!\.\.)\.[\d_]*)?(?:e[+-]?\d[\d_]*)?[ulfi]{0,4}/i,
				lookbehind: true
			}
		],

		'operator': /\|[|=]?|&[&=]?|\+[+=]?|-[-=]?|\.?\.\.|=[>=]?|!(?:i[ns]\b|<>?=?|>=?|=)?|\bi[ns]\b|(?:<[<>]?|>>?>?|\^\^|[*\/%^~])=?/
	});

	Prism.languages.insertBefore('d', 'keyword', {
		'property': /\B@\w*/
	});

	Prism.languages.insertBefore('d', 'function', {
		'register': {
			// Iasm registers
			pattern: /\b(?:[ABCD][LHX]|E[ABCD]X|E?(?:BP|SP|DI|SI)|[ECSDGF]S|CR[0234]|DR[012367]|TR[3-7]|X?MM[0-7]|R[ABCD]X|[BS]PL|R[BS]P|[DS]IL|R[DS]I|R(?:[89]|1[0-5])[BWD]?|XMM(?:[89]|1[0-5])|YMM(?:1[0-5]|\d))\b|\bST(?:\([0-7]\)|\b)/,
			alias: 'variable'
		}
	});

	(function (Prism) {
		var keywords = [
			/\b(?:async|sync|yield)\*/,
			/\b(?:abstract|assert|async|await|break|case|catch|class|const|continue|covariant|default|deferred|do|dynamic|else|enum|export|extension|external|extends|factory|final|finally|for|get|hide|if|implements|interface|import|in|library|mixin|new|null|on|operator|part|rethrow|return|set|show|static|super|switch|sync|this|throw|try|typedef|var|void|while|with|yield)\b/
		];

		// Handles named imports, such as http.Client
		var packagePrefix = /(^|[^\w.])(?:[a-z]\w*\s*\.\s*)*(?:[A-Z]\w*\s*\.\s*)*/.source;

		// based on the dart naming conventions
		var className = {
			pattern: RegExp(packagePrefix + /[A-Z](?:[\d_A-Z]*[a-z]\w*)?\b/.source),
			lookbehind: true,
			inside: {
				'namespace': {
					pattern: /^[a-z]\w*(?:\s*\.\s*[a-z]\w*)*(?:\s*\.)?/,
					inside: {
						'punctuation': /\./
					}
				},
			}
		};

		Prism.languages.dart = Prism.languages.extend('clike', {
			'string': [
				{
					pattern: /r?("""|''')[\s\S]*?\1/,
					greedy: true
				},
				{
					pattern: /r?(["'])(?:\\.|(?!\1)[^\\\r\n])*\1/,
					greedy: true
				}
			],
			'class-name': [
				className,
				{
					// variables and parameters
					// this to support class names (or generic parameters) which do not contain a lower case letter (also works for methods)
					pattern: RegExp(packagePrefix + /[A-Z]\w*(?=\s+\w+\s*[;,=()])/.source),
					lookbehind: true,
					inside: className.inside
				}
			],
			'keyword': keywords,
			'operator': /\bis!|\b(?:as|is)\b|\+\+|--|&&|\|\||<<=?|>>=?|~(?:\/=?)?|[+\-*\/%&^|=!<>]=?|\?/
		});

		Prism.languages.insertBefore('dart', 'function', {
			'metadata': {
				pattern: /@\w+/,
				alias: 'symbol'
			}
		});

		Prism.languages.insertBefore('dart', 'class-name', {
			'generics': {
				pattern: /<(?:[\w\s,.&?]|<(?:[\w\s,.&?]|<(?:[\w\s,.&?]|<[\w\s,.&?]*>)*>)*>)*>/,
				inside: {
					'class-name': className,
					'keyword': keywords,
					'punctuation': /[<>(),.:]/,
					'operator': /[?&|]/
				}
			},
		});
	}(Prism));

	(function (Prism) {

		Prism.languages.diff = {
			'coord': [
				// Match all kinds of coord lines (prefixed by "+++", "---" or "***").
				/^(?:\*{3}|-{3}|\+{3}).*$/m,
				// Match "@@ ... @@" coord lines in unified diff.
				/^@@.*@@$/m,
				// Match coord lines in normal diff (starts with a number).
				/^\d.*$/m
			]

			// deleted, inserted, unchanged, diff
		};

		/**
		 * A map from the name of a block to its line prefix.
		 *
		 * @type {Object<string, string>}
		 */
		var PREFIXES = {
			'deleted-sign': '-',
			'deleted-arrow': '<',
			'inserted-sign': '+',
			'inserted-arrow': '>',
			'unchanged': ' ',
			'diff': '!',
		};

		// add a token for each prefix
		Object.keys(PREFIXES).forEach(function (name) {
			var prefix = PREFIXES[name];

			var alias = [];
			if (!/^\w+$/.test(name)) { // "deleted-sign" -> "deleted"
				alias.push(/\w+/.exec(name)[0]);
			}
			if (name === 'diff') {
				alias.push('bold');
			}

			Prism.languages.diff[name] = {
				pattern: RegExp('^(?:[' + prefix + '].*(?:\r\n?|\n|(?![\\s\\S])))+', 'm'),
				alias: alias,
				inside: {
					'line': {
						pattern: /(.)(?=[\s\S]).*(?:\r\n?|\n)?/,
						lookbehind: true
					},
					'prefix': {
						pattern: /[\s\S]/,
						alias: /\w+/.exec(name)[0]
					}
				}
			};

		});

		// make prefixes available to Diff plugin
		Object.defineProperty(Prism.languages.diff, 'PREFIXES', {
			value: PREFIXES
		});

	}(Prism));

	(function (Prism) {

		// Many of the following regexes will contain negated lookaheads like `[ \t]+(?![ \t])`. This is a trick to ensure
		// that quantifiers behave *atomically*. Atomic quantifiers are necessary to prevent exponential backtracking.

		var spaceAfterBackSlash = /\\[\r\n](?:\s|\\[\r\n]|#.*(?!.))*(?![\s#]|\\[\r\n])/.source;
		// At least one space, comment, or line break
		var space = /(?:[ \t]+(?![ \t])(?:<SP_BS>)?|<SP_BS>)/.source
			.replace(/<SP_BS>/g, function () { return spaceAfterBackSlash; });

		var string = /"(?:[^"\\\r\n]|\\(?:\r\n|[\s\S]))*"|'(?:[^'\\\r\n]|\\(?:\r\n|[\s\S]))*'/.source;
		var option = /--[\w-]+=(?:<STR>|(?!["'])(?:[^\s\\]|\\.)+)/.source.replace(/<STR>/g, function () { return string; });

		var stringRule = {
			pattern: RegExp(string),
			greedy: true
		};
		var commentRule = {
			pattern: /(^[ \t]*)#.*/m,
			lookbehind: true,
			greedy: true
		};

		/**
		 * @param {string} source
		 * @param {string} flags
		 * @returns {RegExp}
		 */
		function re(source, flags) {
			source = source
				.replace(/<OPT>/g, function () { return option; })
				.replace(/<SP>/g, function () { return space; });

			return RegExp(source, flags);
		}

		Prism.languages.docker = {
			'instruction': {
				pattern: /(^[ \t]*)(?:ADD|ARG|CMD|COPY|ENTRYPOINT|ENV|EXPOSE|FROM|HEALTHCHECK|LABEL|MAINTAINER|ONBUILD|RUN|SHELL|STOPSIGNAL|USER|VOLUME|WORKDIR)(?=\s)(?:\\.|[^\r\n\\])*(?:\\$(?:\s|#.*$)*(?![\s#])(?:\\.|[^\r\n\\])*)*/mi,
				lookbehind: true,
				greedy: true,
				inside: {
					'options': {
						pattern: re(/(^(?:ONBUILD<SP>)?\w+<SP>)<OPT>(?:<SP><OPT>)*/.source, 'i'),
						lookbehind: true,
						greedy: true,
						inside: {
							'property': {
								pattern: /(^|\s)--[\w-]+/,
								lookbehind: true
							},
							'string': [
								stringRule,
								{
									pattern: /(=)(?!["'])(?:[^\s\\]|\\.)+/,
									lookbehind: true
								}
							],
							'operator': /\\$/m,
							'punctuation': /=/
						}
					},
					'keyword': [
						{
							// https://docs.docker.com/engine/reference/builder/#healthcheck
							pattern: re(/(^(?:ONBUILD<SP>)?HEALTHCHECK<SP>(?:<OPT><SP>)*)(?:CMD|NONE)\b/.source, 'i'),
							lookbehind: true,
							greedy: true
						},
						{
							// https://docs.docker.com/engine/reference/builder/#from
							pattern: re(/(^(?:ONBUILD<SP>)?FROM<SP>(?:<OPT><SP>)*(?!--)[^ \t\\]+<SP>)AS/.source, 'i'),
							lookbehind: true,
							greedy: true
						},
						{
							// https://docs.docker.com/engine/reference/builder/#onbuild
							pattern: re(/(^ONBUILD<SP>)\w+/.source, 'i'),
							lookbehind: true,
							greedy: true
						},
						{
							pattern: /^\w+/,
							greedy: true
						}
					],
					'comment': commentRule,
					'string': stringRule,
					'variable': /\$(?:\w+|\{[^{}"'\\]*\})/,
					'operator': /\\$/m
				}
			},
			'comment': commentRule
		};

		Prism.languages.dockerfile = Prism.languages.docker;

	}(Prism));

	Prism.languages.ebnf = {
		'comment': /\(\*[\s\S]*?\*\)/,
		'string': {
			pattern: /"[^"\r\n]*"|'[^'\r\n]*'/,
			greedy: true
		},
		'special': {
			pattern: /\?[^?\r\n]*\?/,
			greedy: true,
			alias: 'class-name'
		},

		'definition': {
			pattern: /^(\s*)[a-z]\w*(?:[ \t]+[a-z]\w*)*(?=\s*=)/im,
			lookbehind: true,
			alias: ['rule', 'keyword']
		},
		'rule': /\b[a-z]\w*(?:[ \t]+[a-z]\w*)*\b/i,

		'punctuation': /\([:/]|[:/]\)|[.,;()[\]{}]/,
		'operator': /[-=|*/!]/
	};

	Prism.languages.editorconfig = {
		// https://editorconfig-specification.readthedocs.io/en/latest/
		'comment': /[;#].*/,
		'section': {
			pattern: /(^[ \t]*)\[.+]/m,
			lookbehind: true,
			alias: 'keyword',
			inside: {
				'regex': /\\\\[\[\]{},!?.*]/, // Escape special characters with '\\'
				'operator': /[!?]|\.\.|\*{1,2}/,
				'punctuation': /[\[\]{},]/
			}
		},
		'property': {
			pattern: /(^[ \t]*)[^\s=]+(?=[ \t]*=)/m,
			lookbehind: true
		},
		'value': {
			pattern: /=.*/,
			alias: 'string',
			inside: {
				'punctuation': /^=/
			}
		}
	};

	Prism.languages.elixir = {
		'doc': {
			pattern: /@(?:doc|moduledoc)\s+(?:("""|''')[\s\S]*?\1|("|')(?:\\(?:\r\n|[\s\S])|(?!\2)[^\\\r\n])*\2)/,
			inside: {
				'attribute': /^@\w+/,
				'string': /['"][\s\S]+/
			}
		},
		'comment': {
			pattern: /#.*/m,
			greedy: true
		},
		// ~r"""foo""" (multi-line), ~r'''foo''' (multi-line), ~r/foo/, ~r|foo|, ~r"foo", ~r'foo', ~r(foo), ~r[foo], ~r{foo}, ~r<foo>
		'regex': {
			pattern: /~[rR](?:("""|''')(?:\\[\s\S]|(?!\1)[^\\])+\1|([\/|"'])(?:\\.|(?!\2)[^\\\r\n])+\2|\((?:\\.|[^\\)\r\n])+\)|\[(?:\\.|[^\\\]\r\n])+\]|\{(?:\\.|[^\\}\r\n])+\}|<(?:\\.|[^\\>\r\n])+>)[uismxfr]*/,
			greedy: true
		},
		'string': [
			{
				// ~s"""foo""" (multi-line), ~s'''foo''' (multi-line), ~s/foo/, ~s|foo|, ~s"foo", ~s'foo', ~s(foo), ~s[foo], ~s{foo} (with interpolation care), ~s<foo>
				pattern: /~[cCsSwW](?:("""|''')(?:\\[\s\S]|(?!\1)[^\\])+\1|([\/|"'])(?:\\.|(?!\2)[^\\\r\n])+\2|\((?:\\.|[^\\)\r\n])+\)|\[(?:\\.|[^\\\]\r\n])+\]|\{(?:\\.|#\{[^}]+\}|#(?!\{)|[^#\\}\r\n])+\}|<(?:\\.|[^\\>\r\n])+>)[csa]?/,
				greedy: true,
				inside: {
					// See interpolation below
				}
			},
			{
				pattern: /("""|''')[\s\S]*?\1/,
				greedy: true,
				inside: {
					// See interpolation below
				}
			},
			{
				// Multi-line strings are allowed
				pattern: /("|')(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/,
				greedy: true,
				inside: {
					// See interpolation below
				}
			}
		],
		'atom': {
			// Look-behind prevents bad highlighting of the :: operator
			pattern: /(^|[^:]):\w+/,
			lookbehind: true,
			alias: 'symbol'
		},
		'module': {
			pattern: /\b[A-Z]\w*\b/,
			alias: 'class-name'
		},
		// Look-ahead prevents bad highlighting of the :: operator
		'attr-name': /\w+\??:(?!:)/,
		'argument': {
			// Look-behind prevents bad highlighting of the && operator
			pattern: /(^|[^&])&\d+/,
			lookbehind: true,
			alias: 'variable'
		},
		'attribute': {
			pattern: /@\w+/,
			alias: 'variable'
		},
		'function': /\b[_a-zA-Z]\w*[?!]?(?:(?=\s*(?:\.\s*)?\()|(?=\/\d+))/,
		'number': /\b(?:0[box][a-f\d_]+|\d[\d_]*)(?:\.[\d_]+)?(?:e[+-]?[\d_]+)?\b/i,
		'keyword': /\b(?:after|alias|and|case|catch|cond|def(?:callback|exception|impl|module|p|protocol|struct|delegate)?|do|else|end|fn|for|if|import|not|or|raise|require|rescue|try|unless|use|when)\b/,
		'boolean': /\b(?:true|false|nil)\b/,
		'operator': [
			/\bin\b|&&?|\|[|>]?|\\\\|::|\.\.\.?|\+\+?|-[->]?|<[-=>]|>=|!==?|\B!|=(?:==?|[>~])?|[*\/^]/,
			{
				// We don't want to match <<
				pattern: /([^<])<(?!<)/,
				lookbehind: true
			},
			{
				// We don't want to match >>
				pattern: /([^>])>(?!>)/,
				lookbehind: true
			}
		],
		'punctuation': /<<|>>|[.,%\[\]{}()]/
	};

	Prism.languages.elixir.string.forEach(function (o) {
		o.inside = {
			'interpolation': {
				pattern: /#\{[^}]+\}/,
				inside: {
					'delimiter': {
						pattern: /^#\{|\}$/,
						alias: 'punctuation'
					},
					rest: Prism.languages.elixir
				}
			}
		};
	});

	Prism.languages.elm = {
		'comment': /--.*|{-[\s\S]*?-}/,
		'char': {
			pattern: /'(?:[^\\'\r\n]|\\(?:[abfnrtv\\']|\d+|x[0-9a-fA-F]+))'/,
			greedy: true
		},
		'string': [
			{
				// Multiline strings are wrapped in triple ". Quotes may appear unescaped.
				pattern: /"""[\s\S]*?"""/,
				greedy: true
			},
			{
				pattern: /"(?:[^\\"\r\n]|\\.)*"/,
				greedy: true
			}
		],
		'import-statement': {
			// The imported or hidden names are not included in this import
			// statement. This is because we want to highlight those exactly like
			// we do for the names in the program.
			pattern: /^\s*import\s+[A-Z]\w*(?:\.[A-Z]\w*)*(?:\s+as\s+(?:[A-Z]\w*)(?:\.[A-Z]\w*)*)?(?:\s+exposing\s+)?/m,
			inside: {
				'keyword': /\b(?:import|as|exposing)\b/
			}
		},
		'keyword': /\b(?:alias|as|case|else|exposing|if|in|infixl|infixr|let|module|of|then|type)\b/,
		// These are builtin variables only. Constructors are highlighted later as a constant.
		'builtin': /\b(?:abs|acos|always|asin|atan|atan2|ceiling|clamp|compare|cos|curry|degrees|e|flip|floor|fromPolar|identity|isInfinite|isNaN|logBase|max|min|negate|never|not|pi|radians|rem|round|sin|sqrt|tan|toFloat|toPolar|toString|truncate|turns|uncurry|xor)\b/,
		// decimal integers and floating point numbers | hexadecimal integers
		'number': /\b(?:\d+(?:\.\d+)?(?:e[+-]?\d+)?|0x[0-9a-f]+)\b/i,
		// Most of this is needed because of the meaning of a single '.'.
		// If it stands alone freely, it is the function composition.
		// It may also be a separator between a module name and an identifier => no
		// operator. If it comes together with other special characters it is an
		// operator too.
		// Valid operator characters in 0.18: +-/*=.$<>:&|^?%#@~!
		// Ref: https://groups.google.com/forum/#!msg/elm-dev/0AHSnDdkSkQ/E0SVU70JEQAJ
		'operator': /\s\.\s|[+\-/*=.$<>:&|^?%#@~!]{2,}|[+\-/*=$<>:&|^?%#@~!]/,
		// In Elm, nearly everything is a variable, do not highlight these.
		'hvariable': /\b(?:[A-Z]\w*\.)*[a-z]\w*\b/,
		'constant': /\b(?:[A-Z]\w*\.)*[A-Z]\w*\b/,
		'punctuation': /[{}[\]|(),.:]/
	};

	Prism.languages.erlang = {
		'comment': /%.+/,
		'string': {
			pattern: /"(?:\\.|[^\\"\r\n])*"/,
			greedy: true
		},
		'quoted-function': {
			pattern: /'(?:\\.|[^\\'\r\n])+'(?=\()/,
			alias: 'function'
		},
		'quoted-atom': {
			pattern: /'(?:\\.|[^\\'\r\n])+'/,
			alias: 'atom'
		},
		'boolean': /\b(?:true|false)\b/,
		'keyword': /\b(?:fun|when|case|of|end|if|receive|after|try|catch)\b/,
		'number': [
			/\$\\?./,
			/\d+#[a-z0-9]+/i,
			/(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?/i
		],
		'function': /\b[a-z][\w@]*(?=\()/,
		'variable': {
			// Look-behind is used to prevent wrong highlighting of atoms containing "@"
			pattern: /(^|[^@])(?:\b|\?)[A-Z_][\w@]*/,
			lookbehind: true
		},
		'operator': [
			/[=\/<>:]=|=[:\/]=|\+\+?|--?|[=*\/!]|\b(?:bnot|div|rem|band|bor|bxor|bsl|bsr|not|and|or|xor|orelse|andalso)\b/,
			{
				// We don't want to match <<
				pattern: /(^|[^<])<(?!<)/,
				lookbehind: true
			},
			{
				// We don't want to match >>
				pattern: /(^|[^>])>(?!>)/,
				lookbehind: true
			}
		],
		'atom': /\b[a-z][\w@]*/,
		'punctuation': /[()[\]{}:;,.#|]|<<|>>/

	};

	Prism.languages.fsharp = Prism.languages.extend('clike', {
		'comment': [
			{
				pattern: /(^|[^\\])\(\*(?!\))[\s\S]*?\*\)/,
				lookbehind: true
			},
			{
				pattern: /(^|[^\\:])\/\/.*/,
				lookbehind: true
			}
		],
		'string': {
			pattern: /(?:"""[\s\S]*?"""|@"(?:""|[^"])*"|"(?:\\[\s\S]|[^\\"])*")B?|'(?:[^\\']|\\(?:.|\d{3}|x[a-fA-F\d]{2}|u[a-fA-F\d]{4}|U[a-fA-F\d]{8}))'B?/,
			greedy: true
		},
		'class-name': {
			pattern: /(\b(?:exception|inherit|interface|new|of|type)\s+|\w\s*:\s*|\s:\??>\s*)[.\w]+\b(?:\s*(?:->|\*)\s*[.\w]+\b)*(?!\s*[:.])/,
			lookbehind: true,
			inside: {
				'operator': /->|\*/,
				'punctuation': /\./
			}
		},
		'keyword': /\b(?:let|return|use|yield)(?:!\B|\b)|\b(?:abstract|and|as|assert|base|begin|class|default|delegate|do|done|downcast|downto|elif|else|end|exception|extern|false|finally|for|fun|function|global|if|in|inherit|inline|interface|internal|lazy|match|member|module|mutable|namespace|new|not|null|of|open|or|override|private|public|rec|select|static|struct|then|to|true|try|type|upcast|val|void|when|while|with|asr|land|lor|lsl|lsr|lxor|mod|sig|atomic|break|checked|component|const|constraint|constructor|continue|eager|event|external|fixed|functor|include|method|mixin|object|parallel|process|protected|pure|sealed|tailcall|trait|virtual|volatile)\b/,
		'number': [
			/\b0x[\da-fA-F]+(?:un|lf|LF)?\b/,
			/\b0b[01]+(?:y|uy)?\b/,
			/(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[fm]|e[+-]?\d+)?\b/i,
			/\b\d+(?:[IlLsy]|u[lsy]?|UL)?\b/
		],
		'operator': /([<>~&^])\1\1|([*.:<>&])\2|<-|->|[!=:]=|<?\|{1,3}>?|\??(?:<=|>=|<>|[-+*/%=<>])\??|[!?^&]|~[+~-]|:>|:\?>?/
	});
	Prism.languages.insertBefore('fsharp', 'keyword', {
		'preprocessor': {
			pattern: /^[^\r\n\S]*#.*/m,
			alias: 'property',
			inside: {
				'directive': {
					pattern: /(\s*#)\b(?:else|endif|if|light|line|nowarn)\b/,
					lookbehind: true,
					alias: 'keyword'
				}
			}
		}
	});
	Prism.languages.insertBefore('fsharp', 'punctuation', {
		'computation-expression': {
			pattern: /[_a-z]\w*(?=\s*\{)/i,
			alias: 'keyword'
		}
	});
	Prism.languages.insertBefore('fsharp', 'string', {
		'annotation': {
			pattern: /\[<.+?>\]/,
			inside: {
				'punctuation': /^\[<|>\]$/,
				'class-name': {
					pattern: /^\w+$|(^|;\s*)[A-Z]\w*(?=\()/,
					lookbehind: true
				},
				'annotation-content': {
					pattern: /[\s\S]+/,
					inside: Prism.languages.fsharp
				}
			}
		}
	});

	(function (Prism) {
		Prism.languages.flow = Prism.languages.extend('javascript', {});

		Prism.languages.insertBefore('flow', 'keyword', {
			'type': [
				{
					pattern: /\b(?:[Nn]umber|[Ss]tring|[Bb]oolean|Function|any|mixed|null|void)\b/,
					alias: 'tag'
				}
			]
		});
		Prism.languages.flow['function-variable'].pattern = /(?!\s)[_$a-z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\s*=\s*(?:function\b|(?:\([^()]*\)(?:\s*:\s*\w+)?|(?!\s)[_$a-z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*)\s*=>))/i;
		delete Prism.languages.flow['parameter'];

		Prism.languages.insertBefore('flow', 'operator', {
			'flow-punctuation': {
				pattern: /\{\||\|\}/,
				alias: 'punctuation'
			}
		});

		if (!Array.isArray(Prism.languages.flow.keyword)) {
			Prism.languages.flow.keyword = [Prism.languages.flow.keyword];
		}
		Prism.languages.flow.keyword.unshift(
			{
				pattern: /(^|[^$]\b)(?:type|opaque|declare|Class)\b(?!\$)/,
				lookbehind: true
			},
			{
				pattern: /(^|[^$]\B)\$(?:await|Diff|Exact|Keys|ObjMap|PropertyType|Shape|Record|Supertype|Subtype|Enum)\b(?!\$)/,
				lookbehind: true
			}
		);
	}(Prism));

	Prism.languages.fortran = {
		'quoted-number': {
			pattern: /[BOZ](['"])[A-F0-9]+\1/i,
			alias: 'number'
		},
		'string': {
			pattern: /(?:\w+_)?(['"])(?:\1\1|&(?:\r\n?|\n)(?:[ \t]*!.*(?:\r\n?|\n)|(?![ \t]*!))|(?!\1).)*(?:\1|&)/,
			inside: {
				'comment': {
					pattern: /(&(?:\r\n?|\n)\s*)!.*/,
					lookbehind: true
				}
			}
		},
		'comment': {
			pattern: /!.*/,
			greedy: true
		},
		'boolean': /\.(?:TRUE|FALSE)\.(?:_\w+)?/i,
		'number': /(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[ED][+-]?\d+)?(?:_\w+)?/i,
		'keyword': [
			// Types
			/\b(?:INTEGER|REAL|DOUBLE ?PRECISION|COMPLEX|CHARACTER|LOGICAL)\b/i,
			// END statements
			/\b(?:END ?)?(?:BLOCK ?DATA|DO|FILE|FORALL|FUNCTION|IF|INTERFACE|MODULE(?! PROCEDURE)|PROGRAM|SELECT|SUBROUTINE|TYPE|WHERE)\b/i,
			// Statements
			/\b(?:ALLOCATABLE|ALLOCATE|BACKSPACE|CALL|CASE|CLOSE|COMMON|CONTAINS|CONTINUE|CYCLE|DATA|DEALLOCATE|DIMENSION|DO|END|EQUIVALENCE|EXIT|EXTERNAL|FORMAT|GO ?TO|IMPLICIT(?: NONE)?|INQUIRE|INTENT|INTRINSIC|MODULE PROCEDURE|NAMELIST|NULLIFY|OPEN|OPTIONAL|PARAMETER|POINTER|PRINT|PRIVATE|PUBLIC|READ|RETURN|REWIND|SAVE|SELECT|STOP|TARGET|WHILE|WRITE)\b/i,
			// Others
			/\b(?:ASSIGNMENT|DEFAULT|ELEMENTAL|ELSE|ELSEWHERE|ELSEIF|ENTRY|IN|INCLUDE|INOUT|KIND|NULL|ONLY|OPERATOR|OUT|PURE|RECURSIVE|RESULT|SEQUENCE|STAT|THEN|USE)\b/i
		],
		'operator': [
			/\*\*|\/\/|=>|[=\/]=|[<>]=?|::|[+\-*=%]|\.[A-Z]+\./i,
			{
				// Use lookbehind to prevent confusion with (/ /)
				pattern: /(^|(?!\().)\/(?!\))/,
				lookbehind: true
			}
		],
		'punctuation': /\(\/|\/\)|[(),;:&]/
	};

	Prism.languages.git = {
		/*
		 * A simple one line comment like in a git status command
		 * For instance:
		 * $ git status
		 * # On branch infinite-scroll
		 * # Your branch and 'origin/sharedBranches/frontendTeam/infinite-scroll' have diverged,
		 * # and have 1 and 2 different commits each, respectively.
		 * nothing to commit (working directory clean)
		 */
		'comment': /^#.*/m,

		/*
		 * Regexp to match the changed lines in a git diff output. Check the example below.
		 */
		'deleted': /^[-–].*/m,
		'inserted': /^\+.*/m,

		/*
		 * a string (double and simple quote)
		 */
		'string': /("|')(?:\\.|(?!\1)[^\\\r\n])*\1/m,

		/*
		 * a git command. It starts with a random prompt finishing by a $, then "git" then some other parameters
		 * For instance:
		 * $ git add file.txt
		 */
		'command': {
			pattern: /^.*\$ git .*$/m,
			inside: {
				/*
				 * A git command can contain a parameter starting by a single or a double dash followed by a string
				 * For instance:
				 * $ git diff --cached
				 * $ git log -p
				 */
				'parameter': /\s--?\w+/m
			}
		},

		/*
		 * Coordinates displayed in a git diff command
		 * For instance:
		 * $ git diff
		 * diff --git file.txt file.txt
		 * index 6214953..1d54a52 100644
		 * --- file.txt
		 * +++ file.txt
		 * @@ -1 +1,2 @@
		 * -Here's my tetx file
		 * +Here's my text file
		 * +And this is the second line
		 */
		'coord': /^@@.*@@$/m,

		/*
		 * Match a "commit [SHA1]" line in a git log output.
		 * For instance:
		 * $ git log
		 * commit a11a14ef7e26f2ca62d4b35eac455ce636d0dc09
		 * Author: lgiraudel
		 * Date:   Mon Feb 17 11:18:34 2014 +0100
		 *
		 *     Add of a new line
		 */
		'commit-sha1': /^commit \w{40}$/m
	};

	Prism.languages.glsl = Prism.languages.extend('c', {
		'keyword': /\b(?:attribute|const|uniform|varying|buffer|shared|coherent|volatile|restrict|readonly|writeonly|atomic_uint|layout|centroid|flat|smooth|noperspective|patch|sample|break|continue|do|for|while|switch|case|default|if|else|subroutine|in|out|inout|float|double|int|void|bool|true|false|invariant|precise|discard|return|d?mat[234](?:x[234])?|[ibdu]?vec[234]|uint|lowp|mediump|highp|precision|[iu]?sampler[123]D|[iu]?samplerCube|sampler[12]DShadow|samplerCubeShadow|[iu]?sampler[12]DArray|sampler[12]DArrayShadow|[iu]?sampler2DRect|sampler2DRectShadow|[iu]?samplerBuffer|[iu]?sampler2DMS(?:Array)?|[iu]?samplerCubeArray|samplerCubeArrayShadow|[iu]?image[123]D|[iu]?image2DRect|[iu]?imageCube|[iu]?imageBuffer|[iu]?image[12]DArray|[iu]?imageCubeArray|[iu]?image2DMS(?:Array)?|struct|common|partition|active|asm|class|union|enum|typedef|template|this|resource|goto|inline|noinline|public|static|extern|external|interface|long|short|half|fixed|unsigned|superp|input|output|hvec[234]|fvec[234]|sampler3DRect|filter|sizeof|cast|namespace|using)\b/
	});

	Prism.languages.go = Prism.languages.extend('clike', {
		'string': {
			pattern: /(["'`])(?:\\[\s\S]|(?!\1)[^\\])*\1/,
			greedy: true
		},
		'keyword': /\b(?:break|case|chan|const|continue|default|defer|else|fallthrough|for|func|go(?:to)?|if|import|interface|map|package|range|return|select|struct|switch|type|var)\b/,
		'boolean': /\b(?:_|iota|nil|true|false)\b/,
		'number': /(?:\b0x[a-f\d]+|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[-+]?\d+)?)i?/i,
		'operator': /[*\/%^!=]=?|\+[=+]?|-[=-]?|\|[=|]?|&(?:=|&|\^=?)?|>(?:>=?|=)?|<(?:<=?|=|-)?|:=|\.\.\./,
		'builtin': /\b(?:bool|byte|complex(?:64|128)|error|float(?:32|64)|rune|string|u?int(?:8|16|32|64)?|uintptr|append|cap|close|complex|copy|delete|imag|len|make|new|panic|print(?:ln)?|real|recover)\b/
	});
	delete Prism.languages.go['class-name'];

	Prism.languages.graphql = {
		'comment': /#.*/,
		'description': {
			pattern: /(?:"""(?:[^"]|(?!""")")*"""|"(?:\\.|[^\\"\r\n])*")(?=\s*[a-z_])/i,
			greedy: true,
			alias: 'string',
			inside: {
				'language-markdown': {
					pattern: /(^"(?:"")?)(?!\1)[\s\S]+(?=\1$)/,
					lookbehind: true,
					inside: Prism.languages.markdown
				}
			}
		},
		'string': {
			pattern: /"""(?:[^"]|(?!""")")*"""|"(?:\\.|[^\\"\r\n])*"/,
			greedy: true
		},
		'number': /(?:\B-|\b)\d+(?:\.\d+)?(?:e[+-]?\d+)?\b/i,
		'boolean': /\b(?:true|false)\b/,
		'variable': /\$[a-z_]\w*/i,
		'directive': {
			pattern: /@[a-z_]\w*/i,
			alias: 'function'
		},
		'attr-name': {
			pattern: /[a-z_]\w*(?=\s*(?:\((?:[^()"]|"(?:\\.|[^\\"\r\n])*")*\))?:)/i,
			greedy: true
		},
		'class-name': {
			pattern: /(\b(?:enum|implements|interface|on|scalar|type|union)\s+|&\s*)[a-zA-Z_]\w*/,
			lookbehind: true
		},
		'fragment': {
			pattern: /(\bfragment\s+|\.{3}\s*(?!on\b))[a-zA-Z_]\w*/,
			lookbehind: true,
			alias: 'function'
		},
		'keyword': /\b(?:directive|enum|extend|fragment|implements|input|interface|mutation|on|query|repeatable|scalar|schema|subscription|type|union)\b/,
		'operator': /[!=|&]|\.{3}/,
		'punctuation': /[!(){}\[\]:=,]/,
		'constant': /\b(?!ID\b)[A-Z][A-Z_\d]*\b/
	};

	Prism.languages.haskell = {
		'comment': {
			pattern: /(^|[^-!#$%*+=?&@|~.:<>^\\\/])(?:--(?:(?=.)[^-!#$%*+=?&@|~.:<>^\\\/].*|$)|{-[\s\S]*?-})/m,
			lookbehind: true
		},
		'char': {
			pattern: /'(?:[^\\']|\\(?:[abfnrtv\\"'&]|\^[A-Z@[\]^_]|NUL|SOH|STX|ETX|EOT|ENQ|ACK|BEL|BS|HT|LF|VT|FF|CR|SO|SI|DLE|DC1|DC2|DC3|DC4|NAK|SYN|ETB|CAN|EM|SUB|ESC|FS|GS|RS|US|SP|DEL|\d+|o[0-7]+|x[0-9a-fA-F]+))'/,
			alias: 'string'
		},
		'string': {
			pattern: /"(?:[^\\"]|\\(?:\S|\s+\\))*"/,
			greedy: true
		},
		'keyword': /\b(?:case|class|data|deriving|do|else|if|in|infixl|infixr|instance|let|module|newtype|of|primitive|then|type|where)\b/,
		'import-statement': {
			// The imported or hidden names are not included in this import
			// statement. This is because we want to highlight those exactly like
			// we do for the names in the program.
			pattern: /(^\s*)import\s+(?:qualified\s+)?(?:[A-Z][\w']*)(?:\.[A-Z][\w']*)*(?:\s+as\s+(?:[A-Z][\w']*)(?:\.[A-Z][\w']*)*)?(?:\s+hiding\b)?/m,
			lookbehind: true,
			inside: {
				'keyword': /\b(?:import|qualified|as|hiding)\b/
			}
		},
		// These are builtin variables only. Constructors are highlighted later as a constant.
		'builtin': /\b(?:abs|acos|acosh|all|and|any|appendFile|approxRational|asTypeOf|asin|asinh|atan|atan2|atanh|basicIORun|break|catch|ceiling|chr|compare|concat|concatMap|const|cos|cosh|curry|cycle|decodeFloat|denominator|digitToInt|div|divMod|drop|dropWhile|either|elem|encodeFloat|enumFrom|enumFromThen|enumFromThenTo|enumFromTo|error|even|exp|exponent|fail|filter|flip|floatDigits|floatRadix|floatRange|floor|fmap|foldl|foldl1|foldr|foldr1|fromDouble|fromEnum|fromInt|fromInteger|fromIntegral|fromRational|fst|gcd|getChar|getContents|getLine|group|head|id|inRange|index|init|intToDigit|interact|ioError|isAlpha|isAlphaNum|isAscii|isControl|isDenormalized|isDigit|isHexDigit|isIEEE|isInfinite|isLower|isNaN|isNegativeZero|isOctDigit|isPrint|isSpace|isUpper|iterate|last|lcm|length|lex|lexDigits|lexLitChar|lines|log|logBase|lookup|map|mapM|mapM_|max|maxBound|maximum|maybe|min|minBound|minimum|mod|negate|not|notElem|null|numerator|odd|or|ord|otherwise|pack|pi|pred|primExitWith|print|product|properFraction|putChar|putStr|putStrLn|quot|quotRem|range|rangeSize|read|readDec|readFile|readFloat|readHex|readIO|readInt|readList|readLitChar|readLn|readOct|readParen|readSigned|reads|readsPrec|realToFrac|recip|rem|repeat|replicate|return|reverse|round|scaleFloat|scanl|scanl1|scanr|scanr1|seq|sequence|sequence_|show|showChar|showInt|showList|showLitChar|showParen|showSigned|showString|shows|showsPrec|significand|signum|sin|sinh|snd|sort|span|splitAt|sqrt|subtract|succ|sum|tail|take|takeWhile|tan|tanh|threadToIOResult|toEnum|toInt|toInteger|toLower|toRational|toUpper|truncate|uncurry|undefined|unlines|until|unwords|unzip|unzip3|userError|words|writeFile|zip|zip3|zipWith|zipWith3)\b/,
		// decimal integers and floating point numbers | octal integers | hexadecimal integers
		'number': /\b(?:\d+(?:\.\d+)?(?:e[+-]?\d+)?|0o[0-7]+|0x[0-9a-f]+)\b/i,
		// Most of this is needed because of the meaning of a single '.'.
		// If it stands alone freely, it is the function composition.
		// It may also be a separator between a module name and an identifier => no
		// operator. If it comes together with other special characters it is an
		// operator too.
		'operator': /\s\.\s|[-!#$%*+=?&@|~:<>^\\\/]*\.[-!#$%*+=?&@|~.:<>^\\\/]+|[-!#$%*+=?&@|~.:<>^\\\/]+\.[-!#$%*+=?&@|~:<>^\\\/]*|[-!#$%*+=?&@|~:<>^\\\/]+|`(?:[A-Z][\w']*\.)*[_a-z][\w']*`/,
		// In Haskell, nearly everything is a variable, do not highlight these.
		'hvariable': /\b(?:[A-Z][\w']*\.)*[_a-z][\w']*\b/,
		'constant': /\b(?:[A-Z][\w']*\.)*[A-Z][\w']*\b/,
		'punctuation': /[{}[\];(),.:]/
	};

	Prism.languages.hs = Prism.languages.haskell;

	Prism.languages.hcl = {
		'comment': /(?:\/\/|#).*|\/\*[\s\S]*?(?:\*\/|$)/,
		'heredoc': {
			pattern: /<<-?(\w+\b)[\s\S]*?^[ \t]*\1/m,
			greedy: true,
			alias: 'string'
		},
		'keyword': [
			{
				pattern: /(?:resource|data)\s+(?:"(?:\\[\s\S]|[^\\"])*")(?=\s+"[\w-]+"\s+{)/i,
				inside: {
					'type': {
						pattern: /(resource|data|\s+)(?:"(?:\\[\s\S]|[^\\"])*")/i,
						lookbehind: true,
						alias: 'variable'
					}
				}
			},
			{
				pattern: /(?:provider|provisioner|variable|output|module|backend)\s+(?:[\w-]+|"(?:\\[\s\S]|[^\\"])*")\s+(?={)/i,
				inside: {
					'type': {
						pattern: /(provider|provisioner|variable|output|module|backend)\s+(?:[\w-]+|"(?:\\[\s\S]|[^\\"])*")\s+/i,
						lookbehind: true,
						alias: 'variable'
					}
				}
			},
			/[\w-]+(?=\s+{)/
		],
		'property': [
			/[\w-\.]+(?=\s*=(?!=))/,
			/"(?:\\[\s\S]|[^\\"])+"(?=\s*[:=])/,
		],
		'string': {
			pattern: /"(?:[^\\$"]|\\[\s\S]|\$(?:(?=")|\$+(?!\$)|[^"${])|\$\{(?:[^{}"]|"(?:[^\\"]|\\[\s\S])*")*\})*"/,
			greedy: true,
			inside: {
				'interpolation': {
					pattern: /(^|[^$])\$\{(?:[^{}"]|"(?:[^\\"]|\\[\s\S])*")*\}/,
					lookbehind: true,
					inside: {
						'type': {
							pattern: /(\b(?:terraform|var|self|count|module|path|data|local)\b\.)[\w\*]+/i,
							lookbehind: true,
							alias: 'variable'
						},
						'keyword': /\b(?:terraform|var|self|count|module|path|data|local)\b/i,
						'function': /\w+(?=\()/,
						'string': {
							pattern: /"(?:\\[\s\S]|[^\\"])*"/,
							greedy: true,
						},
						'number': /\b0x[\da-f]+\b|\b\d+(?:\.\d*)?(?:e[+-]?\d+)?/i,
						'punctuation': /[!\$#%&'()*+,.\/;<=>@\[\\\]^`{|}~?:]/,
					}
				},
			}
		},
		'number': /\b0x[\da-f]+\b|\b\d+(?:\.\d*)?(?:e[+-]?\d+)?/i,
		'boolean': /\b(?:true|false)\b/i,
		'punctuation': /[=\[\]{}]/,
	};

	Prism.languages.hlsl = Prism.languages.extend('c', {

		// Regarding keywords and class names:
		// The list of all keywords was split into 'keyword' and 'class-name' tokens based on whether they are capitalized.
		// https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-appendix-keywords
		// https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-appendix-reserved-words
		'class-name': [
			Prism.languages.c['class-name'],
			/\b(?:AppendStructuredBuffer|BlendState|Buffer|ByteAddressBuffer|CompileShader|ComputeShader|ConsumeStructuredBuffer|DepthStencilState|DepthStencilView|DomainShader|GeometryShader|Hullshader|InputPatch|LineStream|OutputPatch|PixelShader|PointStream|RasterizerState|RenderTargetView|RWBuffer|RWByteAddressBuffer|RWStructuredBuffer|RWTexture(?:1D|1DArray|2D|2DArray|3D)|SamplerComparisonState|SamplerState|StructuredBuffer|Texture(?:1D|1DArray|2D|2DArray|2DMS|2DMSArray|3D|Cube|CubeArray)|TriangleStream|VertexShader)\b/
		],
		'keyword': [
			// HLSL keyword
			/\b(?:asm|asm_fragment|auto|break|case|catch|cbuffer|centroid|char|class|column_major|compile|compile_fragment|const|const_cast|continue|default|delete|discard|do|dynamic_cast|else|enum|explicit|export|extern|for|friend|fxgroup|goto|groupshared|if|in|inline|inout|interface|line|lineadj|linear|long|matrix|mutable|namespace|new|nointerpolation|noperspective|operator|out|packoffset|pass|pixelfragment|point|precise|private|protected|public|register|reinterpret_cast|return|row_major|sample|sampler|shared|short|signed|sizeof|snorm|stateblock|stateblock_state|static|static_cast|string|struct|switch|tbuffer|technique|technique10|technique11|template|texture|this|throw|triangle|triangleadj|try|typedef|typename|uniform|union|unorm|unsigned|using|vector|vertexfragment|virtual|void|volatile|while)\b/,
			// scalar, vector, and matrix types
			/\b(?:bool|double|dword|float|half|int|min(?:10float|12int|16(?:float|int|uint))|uint)(?:[1-4](?:x[1-4])?)?\b/
		],
		// https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-appendix-grammar#floating-point-numbers
		'number': /(?:(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[eE][+-]?\d+)?|\b0x[\da-fA-F]+)[fFhHlLuU]?\b/,
		'boolean': /\b(?:false|true)\b/
	});

	(function (Prism) {
		Prism.languages.http = {
			'request-line': {
				pattern: /^(?:GET|HEAD|POST|PUT|DELETE|CONNECT|OPTIONS|TRACE|PATCH|PRI|SEARCH)\s(?:https?:\/\/|\/)\S*\sHTTP\/[0-9.]+/m,
				inside: {
					// HTTP Method
					'method': {
						pattern: /^[A-Z]+\b/,
						alias: 'property'
					},
					// Request Target e.g. http://example.com, /path/to/file
					'request-target': {
						pattern: /^(\s)(?:https?:\/\/|\/)\S*(?=\s)/,
						lookbehind: true,
						alias: 'url',
						inside: Prism.languages.uri
					},
					// HTTP Version
					'http-version': {
						pattern: /^(\s)HTTP\/[0-9.]+/,
						lookbehind: true,
						alias: 'property'
					},
				}
			},
			'response-status': {
				pattern: /^HTTP\/[0-9.]+ \d+ .+/m,
				inside: {
					// HTTP Version
					'http-version': {
						pattern: /^HTTP\/[0-9.]+/,
						alias: 'property'
					},
					// Status Code
					'status-code': {
						pattern: /^(\s)\d+(?=\s)/,
						lookbehind: true,
						alias: 'number'
					},
					// Reason Phrase
					'reason-phrase': {
						pattern: /^(\s).+/,
						lookbehind: true,
						alias: 'string'
					}
				}
			},
			// HTTP header name
			'header-name': {
				pattern: /^[\w-]+:(?=.)/m,
				alias: 'keyword'
			}
		};

		// Create a mapping of Content-Type headers to language definitions
		var langs = Prism.languages;
		var httpLanguages = {
			'application/javascript': langs.javascript,
			'application/json': langs.json || langs.javascript,
			'application/xml': langs.xml,
			'text/xml': langs.xml,
			'text/html': langs.html,
			'text/css': langs.css
		};

		// Declare which types can also be suffixes
		var suffixTypes = {
			'application/json': true,
			'application/xml': true
		};

		/**
		 * Returns a pattern for the given content type which matches it and any type which has it as a suffix.
		 *
		 * @param {string} contentType
		 * @returns {string}
		 */
		function getSuffixPattern(contentType) {
			var suffix = contentType.replace(/^[a-z]+\//, '');
			var suffixPattern = '\\w+/(?:[\\w.-]+\\+)+' + suffix + '(?![+\\w.-])';
			return '(?:' + contentType + '|' + suffixPattern + ')';
		}

		// Insert each content type parser that has its associated language
		// currently loaded.
		var options;
		for (var contentType in httpLanguages) {
			if (httpLanguages[contentType]) {
				options = options || {};

				var pattern = suffixTypes[contentType] ? getSuffixPattern(contentType) : contentType;
				options[contentType.replace(/\//g, '-')] = {
					pattern: RegExp('(content-type:\\s*' + pattern + '(?:(?:\\r\\n?|\\n).+)*)(?:\\r?\\n|\\r){2}[\\s\\S]*', 'i'),
					lookbehind: true,
					inside: httpLanguages[contentType]
				};
			}
		}
		if (options) {
			Prism.languages.insertBefore('http', 'header-name', options);
		}

	}(Prism));

	/**
	 * Original by Scott Helme.
	 *
	 * Reference: https://scotthelme.co.uk/hpkp-cheat-sheet/
	 */

	Prism.languages.hpkp = {
		'directive': {
			pattern: /\b(?:(?:includeSubDomains|preload|strict)(?: |;)|pin-sha256="[a-zA-Z\d+=/]+"|(?:max-age|report-uri)=|report-to )/,
			alias: 'keyword'
		},
		'safe': {
			pattern: /\b\d{7,}\b/,
			alias: 'selector'
		},
		'unsafe': {
			pattern: /\b\d{1,6}\b/,
			alias: 'function'
		}
	};

	/**
	 * Original by Scott Helme.
	 *
	 * Reference: https://scotthelme.co.uk/hsts-cheat-sheet/
	 */

	Prism.languages.hsts = {
		'directive': {
			pattern: /\b(?:max-age=|includeSubDomains|preload)/,
			alias: 'keyword'
		},
		'safe': {
			pattern: /\b\d{8,}\b/,
			alias: 'selector'
		},
		'unsafe': {
			pattern: /\b\d{1,7}\b/,
			alias: 'function'
		}
	};

	(function (Prism) {
		Prism.languages.ignore = {
			// https://git-scm.com/docs/gitignore
			'comment': /^#.*/m,
			'entry': {
				pattern: /\S(?:.*(?:(?:\\ )|\S))?/,
				alias: 'string',
				inside: {
					'operator': /^!|\*\*?|\?/,
					'regex': {
						pattern: /(^|[^\\])\[[^\[\]]*\]/,
						lookbehind: true
					},
					'punctuation': /\//
				}
			}
		};

		Prism.languages.gitignore = Prism.languages.ignore;
		Prism.languages.hgignore = Prism.languages.ignore;
		Prism.languages.npmignore = Prism.languages.ignore;

	}(Prism));

	Prism.languages.ini = {

		/**
		 * The component mimics the behavior of the Win32 API parser.
		 *
		 * @see {@link https://github.com/PrismJS/prism/issues/2775#issuecomment-787477723}
		 */

		'comment': {
			pattern: /(^[ \f\t\v]*)[#;][^\n\r]*/m,
			lookbehind: true
		},
		'header': {
			pattern: /(^[ \f\t\v]*)\[[^\n\r\]]*\]?/m,
			lookbehind: true,
			inside: {
				'section-name': {
					pattern: /(^\[[ \f\t\v]*)[^ \f\t\v\]]+(?:[ \f\t\v]+[^ \f\t\v\]]+)*/,
					lookbehind: true,
					alias: 'selector'
				},
				'punctuation': /\[|\]/
			}
		},
		'key': {
			pattern: /(^[ \f\t\v]*)[^ \f\n\r\t\v=]+(?:[ \f\t\v]+[^ \f\n\r\t\v=]+)*(?=[ \f\t\v]*=)/m,
			lookbehind: true,
			alias: 'attr-name'
		},
		'value': {
			pattern: /(=[ \f\t\v]*)[^ \f\n\r\t\v]+(?:[ \f\t\v]+[^ \f\n\r\t\v]+)*/,
			lookbehind: true,
			alias: 'attr-value',
			inside: {
				'inner-value': {
					pattern: /^("|').+(?=\1$)/,
					lookbehind: true
				}
			}
		},
		'punctuation': /=/
	};

	(function (Prism) {

		var keywords = /\b(?:abstract|assert|boolean|break|byte|case|catch|char|class|const|continue|default|do|double|else|enum|exports|extends|final|finally|float|for|goto|if|implements|import|instanceof|int|interface|long|module|native|new|non-sealed|null|open|opens|package|permits|private|protected|provides|public|record|requires|return|sealed|short|static|strictfp|super|switch|synchronized|this|throw|throws|to|transient|transitive|try|uses|var|void|volatile|while|with|yield)\b/;

		// full package (optional) + parent classes (optional)
		var classNamePrefix = /(^|[^\w.])(?:[a-z]\w*\s*\.\s*)*(?:[A-Z]\w*\s*\.\s*)*/.source;

		// based on the java naming conventions
		var className = {
			pattern: RegExp(classNamePrefix + /[A-Z](?:[\d_A-Z]*[a-z]\w*)?\b/.source),
			lookbehind: true,
			inside: {
				'namespace': {
					pattern: /^[a-z]\w*(?:\s*\.\s*[a-z]\w*)*(?:\s*\.)?/,
					inside: {
						'punctuation': /\./
					}
				},
				'punctuation': /\./
			}
		};

		Prism.languages.java = Prism.languages.extend('clike', {
			'class-name': [
				className,
				{
					// variables and parameters
					// this to support class names (or generic parameters) which do not contain a lower case letter (also works for methods)
					pattern: RegExp(classNamePrefix + /[A-Z]\w*(?=\s+\w+\s*[;,=())])/.source),
					lookbehind: true,
					inside: className.inside
				}
			],
			'keyword': keywords,
			'function': [
				Prism.languages.clike.function,
				{
					pattern: /(\:\:\s*)[a-z_]\w*/,
					lookbehind: true
				}
			],
			'number': /\b0b[01][01_]*L?\b|\b0x(?:\.[\da-f_p+-]+|[\da-f_]+(?:\.[\da-f_p+-]+)?)\b|(?:\b\d[\d_]*(?:\.[\d_]*)?|\B\.\d[\d_]*)(?:e[+-]?\d[\d_]*)?[dfl]?/i,
			'operator': {
				pattern: /(^|[^.])(?:<<=?|>>>?=?|->|--|\+\+|&&|\|\||::|[?:~]|[-+*/%&|^!=<>]=?)/m,
				lookbehind: true
			}
		});

		Prism.languages.insertBefore('java', 'string', {
			'triple-quoted-string': {
				// http://openjdk.java.net/jeps/355#Description
				pattern: /"""[ \t]*[\r\n](?:(?:"|"")?(?:\\.|[^"\\]))*"""/,
				greedy: true,
				alias: 'string'
			}
		});

		Prism.languages.insertBefore('java', 'class-name', {
			'annotation': {
				pattern: /(^|[^.])@\w+(?:\s*\.\s*\w+)*/,
				lookbehind: true,
				alias: 'punctuation'
			},
			'generics': {
				pattern: /<(?:[\w\s,.?]|&(?!&)|<(?:[\w\s,.?]|&(?!&)|<(?:[\w\s,.?]|&(?!&)|<(?:[\w\s,.?]|&(?!&))*>)*>)*>)*>/,
				inside: {
					'class-name': className,
					'keyword': keywords,
					'punctuation': /[<>(),.:]/,
					'operator': /[?&|]/
				}
			},
			'namespace': {
				pattern: RegExp(
					/(\b(?:exports|import(?:\s+static)?|module|open|opens|package|provides|requires|to|transitive|uses|with)\s+)(?!<keyword>)[a-z]\w*(?:\.[a-z]\w*)*\.?/
						.source.replace(/<keyword>/g, function () { return keywords.source; })),
				lookbehind: true,
				inside: {
					'punctuation': /\./,
				}
			}
		});
	}(Prism));

	(function (Prism) {

		/**
		 * Returns the placeholder for the given language id and index.
		 *
		 * @param {string} language
		 * @param {string|number} index
		 * @returns {string}
		 */
		function getPlaceholder(language, index) {
			return '___' + language.toUpperCase() + index + '___';
		}

		Object.defineProperties(Prism.languages['markup-templating'] = {}, {
			buildPlaceholders: {
				/**
				 * Tokenize all inline templating expressions matching `placeholderPattern`.
				 *
				 * If `replaceFilter` is provided, only matches of `placeholderPattern` for which `replaceFilter` returns
				 * `true` will be replaced.
				 *
				 * @param {object} env The environment of the `before-tokenize` hook.
				 * @param {string} language The language id.
				 * @param {RegExp} placeholderPattern The matches of this pattern will be replaced by placeholders.
				 * @param {(match: string) => boolean} [replaceFilter]
				 */
				value: function (env, language, placeholderPattern, replaceFilter) {
					if (env.language !== language) {
						return;
					}

					var tokenStack = env.tokenStack = [];

					env.code = env.code.replace(placeholderPattern, function (match) {
						if (typeof replaceFilter === 'function' && !replaceFilter(match)) {
							return match;
						}
						var i = tokenStack.length;
						var placeholder;

						// Check for existing strings
						while (env.code.indexOf(placeholder = getPlaceholder(language, i)) !== -1)
							++i;

						// Create a sparse array
						tokenStack[i] = match;

						return placeholder;
					});

					// Switch the grammar to markup
					env.grammar = Prism.languages.markup;
				}
			},
			tokenizePlaceholders: {
				/**
				 * Replace placeholders with proper tokens after tokenizing.
				 *
				 * @param {object} env The environment of the `after-tokenize` hook.
				 * @param {string} language The language id.
				 */
				value: function (env, language) {
					if (env.language !== language || !env.tokenStack) {
						return;
					}

					// Switch the grammar back
					env.grammar = Prism.languages[language];

					var j = 0;
					var keys = Object.keys(env.tokenStack);

					function walkTokens(tokens) {
						for (var i = 0; i < tokens.length; i++) {
							// all placeholders are replaced already
							if (j >= keys.length) {
								break;
							}

							var token = tokens[i];
							if (typeof token === 'string' || (token.content && typeof token.content === 'string')) {
								var k = keys[j];
								var t = env.tokenStack[k];
								var s = typeof token === 'string' ? token : token.content;
								var placeholder = getPlaceholder(language, k);

								var index = s.indexOf(placeholder);
								if (index > -1) {
									++j;

									var before = s.substring(0, index);
									var middle = new Prism.Token(language, Prism.tokenize(t, env.grammar), 'language-' + language, t);
									var after = s.substring(index + placeholder.length);

									var replacement = [];
									if (before) {
										replacement.push.apply(replacement, walkTokens([before]));
									}
									replacement.push(middle);
									if (after) {
										replacement.push.apply(replacement, walkTokens([after]));
									}

									if (typeof token === 'string') {
										tokens.splice.apply(tokens, [i, 1].concat(replacement));
									} else {
										token.content = replacement;
									}
								}
							} else if (token.content /* && typeof token.content !== 'string' */) {
								walkTokens(token.content);
							}
						}

						return tokens;
					}

					walkTokens(env.tokens);
				}
			}
		});

	}(Prism));

	/**
	 * Original by Aaron Harun: http://aahacreative.com/2012/07/31/php-syntax-highlighting-prism/
	 * Modified by Miles Johnson: http://milesj.me
	 * Rewritten by Tom Pavelec
	 *
	 * Supports PHP 5.3 - 8.0
	 */
	(function (Prism) {
		var comment = /\/\*[\s\S]*?\*\/|\/\/.*|#(?!\[).*/;
		var constant = [
			{
				pattern: /\b(?:false|true)\b/i,
				alias: 'boolean'
			},
			{
				pattern: /(::\s*)\b[a-z_]\w*\b(?!\s*\()/i,
				greedy: true,
				lookbehind: true,
			},
			{
				pattern: /(\b(?:case|const)\s+)\b[a-z_]\w*(?=\s*[;=])/i,
				greedy: true,
				lookbehind: true,
			},
			/\b(?:null)\b/i,
			/\b[A-Z_][A-Z0-9_]*\b(?!\s*\()/,
		];
		var number = /\b0b[01]+(?:_[01]+)*\b|\b0o[0-7]+(?:_[0-7]+)*\b|\b0x[\da-f]+(?:_[\da-f]+)*\b|(?:\b\d+(?:_\d+)*\.?(?:\d+(?:_\d+)*)?|\B\.\d+)(?:e[+-]?\d+)?/i;
		var operator = /<?=>|\?\?=?|\.{3}|\??->|[!=]=?=?|::|\*\*=?|--|\+\+|&&|\|\||<<|>>|[?~]|[/^|%*&<>.+-]=?/;
		var punctuation = /[{}\[\](),:;]/;

		Prism.languages.php = {
			'delimiter': {
				pattern: /\?>$|^<\?(?:php(?=\s)|=)?/i,
				alias: 'important'
			},
			'comment': comment,
			'variable': /\$+(?:\w+\b|(?={))/i,
			'package': {
				pattern: /(namespace\s+|use\s+(?:function\s+)?)(?:\\?\b[a-z_]\w*)+\b(?!\\)/i,
				lookbehind: true,
				inside: {
					'punctuation': /\\/
				}
			},
			'class-name-definition': {
				pattern: /(\b(?:class|enum|interface|trait)\s+)\b[a-z_]\w*(?!\\)\b/i,
				lookbehind: true,
				alias: 'class-name'
			},
			'function-definition': {
				pattern: /(\bfunction\s+)[a-z_]\w*(?=\s*\()/i,
				lookbehind: true,
				alias: 'function'
			},
			'keyword': [
				{
					pattern: /(\(\s*)\b(?:bool|boolean|int|integer|float|string|object|array)\b(?=\s*\))/i,
					alias: 'type-casting',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /([(,?]\s*)\b(?:bool|int|float|string|object|array(?!\s*\()|mixed|self|static|callable|iterable|(?:null|false)(?=\s*\|))\b(?=\s*\$)/i,
					alias: 'type-hint',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /([(,?]\s*[a-z0-9_|]\|\s*)(?:null|false)\b(?=\s*\$)/i,
					alias: 'type-hint',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /(\)\s*:\s*(?:\?\s*)?)\b(?:bool|int|float|string|object|void|array(?!\s*\()|mixed|self|static|callable|iterable|(?:null|false)(?=\s*\|))\b/i,
					alias: 'return-type',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /(\)\s*:\s*(?:\?\s*)?[a-z0-9_|]\|\s*)(?:null|false)\b/i,
					alias: 'return-type',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /\b(?:bool|int|float|string|object|void|array(?!\s*\()|mixed|iterable|(?:null|false)(?=\s*\|))\b/i,
					alias: 'type-declaration',
					greedy: true
				},
				{
					pattern: /(\|\s*)(?:null|false)\b/i,
					alias: 'type-declaration',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /\b(?:parent|self|static)(?=\s*::)/i,
					alias: 'static-context',
					greedy: true
				},
				{
					// yield from
					pattern: /(\byield\s+)from\b/i,
					lookbehind: true
				},
				// `class` is always a keyword unlike other keywords
				/\bclass\b/i,
				{
					// https://www.php.net/manual/en/reserved.keywords.php
					//
					// keywords cannot be preceded by "->"
					// the complex lookbehind means `(?<!(?:->|::)\s*)`
					pattern: /((?:^|[^\s>:]|(?:^|[^-])>|(?:^|[^:]):)\s*)\b(?:__halt_compiler|abstract|and|array|as|break|callable|case|catch|clone|const|continue|declare|default|die|do|echo|else|elseif|empty|enddeclare|endfor|endforeach|endif|endswitch|endwhile|enum|eval|exit|extends|final|finally|fn|for|foreach|function|global|goto|if|implements|include|include_once|instanceof|insteadof|interface|isset|list|namespace|match|new|or|parent|print|private|protected|public|require|require_once|return|self|static|switch|throw|trait|try|unset|use|var|while|xor|yield)\b/i,
					lookbehind: true
				}
			],
			'argument-name': {
				pattern: /([(,]\s+)\b[a-z_]\w*(?=\s*:(?!:))/i,
				lookbehind: true
			},
			'class-name': [
				{
					pattern: /(\b(?:extends|implements|instanceof|new(?!\s+self|\s+static))\s+|\bcatch\s*\()\b[a-z_]\w*(?!\\)\b/i,
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /(\|\s*)\b[a-z_]\w*(?!\\)\b/i,
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /\b[a-z_]\w*(?!\\)\b(?=\s*\|)/i,
					greedy: true
				},
				{
					pattern: /(\|\s*)(?:\\?\b[a-z_]\w*)+\b/i,
					alias: 'class-name-fully-qualified',
					greedy: true,
					lookbehind: true,
					inside: {
						'punctuation': /\\/
					}
				},
				{
					pattern: /(?:\\?\b[a-z_]\w*)+\b(?=\s*\|)/i,
					alias: 'class-name-fully-qualified',
					greedy: true,
					inside: {
						'punctuation': /\\/
					}
				},
				{
					pattern: /(\b(?:extends|implements|instanceof|new(?!\s+self\b|\s+static\b))\s+|\bcatch\s*\()(?:\\?\b[a-z_]\w*)+\b(?!\\)/i,
					alias: 'class-name-fully-qualified',
					greedy: true,
					lookbehind: true,
					inside: {
						'punctuation': /\\/
					}
				},
				{
					pattern: /\b[a-z_]\w*(?=\s*\$)/i,
					alias: 'type-declaration',
					greedy: true
				},
				{
					pattern: /(?:\\?\b[a-z_]\w*)+(?=\s*\$)/i,
					alias: ['class-name-fully-qualified', 'type-declaration'],
					greedy: true,
					inside: {
						'punctuation': /\\/
					}
				},
				{
					pattern: /\b[a-z_]\w*(?=\s*::)/i,
					alias: 'static-context',
					greedy: true
				},
				{
					pattern: /(?:\\?\b[a-z_]\w*)+(?=\s*::)/i,
					alias: ['class-name-fully-qualified', 'static-context'],
					greedy: true,
					inside: {
						'punctuation': /\\/
					}
				},
				{
					pattern: /([(,?]\s*)[a-z_]\w*(?=\s*\$)/i,
					alias: 'type-hint',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /([(,?]\s*)(?:\\?\b[a-z_]\w*)+(?=\s*\$)/i,
					alias: ['class-name-fully-qualified', 'type-hint'],
					greedy: true,
					lookbehind: true,
					inside: {
						'punctuation': /\\/
					}
				},
				{
					pattern: /(\)\s*:\s*(?:\?\s*)?)\b[a-z_]\w*(?!\\)\b/i,
					alias: 'return-type',
					greedy: true,
					lookbehind: true
				},
				{
					pattern: /(\)\s*:\s*(?:\?\s*)?)(?:\\?\b[a-z_]\w*)+\b(?!\\)/i,
					alias: ['class-name-fully-qualified', 'return-type'],
					greedy: true,
					lookbehind: true,
					inside: {
						'punctuation': /\\/
					}
				}
			],
			'constant': constant,
			'function': /\b\w+(?=\s*\()/,
			'property': {
				pattern: /(->\s*)\w+/,
				lookbehind: true
			},
			'number': number,
			'operator': operator,
			'punctuation': punctuation
		};

		var string_interpolation = {
			pattern: /{\$(?:{(?:{[^{}]+}|[^{}]+)}|[^{}])+}|(^|[^\\{])\$+(?:\w+(?:\[[^\r\n\[\]]+\]|->\w+)?)/,
			lookbehind: true,
			inside: Prism.languages.php
		};

		var string = [
			{
				pattern: /<<<'([^']+)'[\r\n](?:.*[\r\n])*?\1;/,
				alias: 'nowdoc-string',
				greedy: true,
				inside: {
					'delimiter': {
						pattern: /^<<<'[^']+'|[a-z_]\w*;$/i,
						alias: 'symbol',
						inside: {
							'punctuation': /^<<<'?|[';]$/
						}
					}
				}
			},
			{
				pattern: /<<<(?:"([^"]+)"[\r\n](?:.*[\r\n])*?\1;|([a-z_]\w*)[\r\n](?:.*[\r\n])*?\2;)/i,
				alias: 'heredoc-string',
				greedy: true,
				inside: {
					'delimiter': {
						pattern: /^<<<(?:"[^"]+"|[a-z_]\w*)|[a-z_]\w*;$/i,
						alias: 'symbol',
						inside: {
							'punctuation': /^<<<"?|[";]$/
						}
					},
					'interpolation': string_interpolation
				}
			},
			{
				pattern: /`(?:\\[\s\S]|[^\\`])*`/,
				alias: 'backtick-quoted-string',
				greedy: true
			},
			{
				pattern: /'(?:\\[\s\S]|[^\\'])*'/,
				alias: 'single-quoted-string',
				greedy: true
			},
			{
				pattern: /"(?:\\[\s\S]|[^\\"])*"/,
				alias: 'double-quoted-string',
				greedy: true,
				inside: {
					'interpolation': string_interpolation
				}
			}
		];

		Prism.languages.insertBefore('php', 'variable', {
			'string': string,
			'attribute': {
				pattern: /#\[(?:[^"'\/#]|\/(?![*/])|\/\/.*$|#(?!\[).*$|\/\*(?:[^*]|\*(?!\/))*\*\/|"(?:\\[\s\S]|[^\\"])*"|'(?:\\[\s\S]|[^\\'])*')+\](?=\s*[a-z$#])/mi,
				greedy: true,
				inside: {
					'attribute-content': {
						pattern: /^(#\[)[\s\S]+(?=]$)/,
						lookbehind: true,
						// inside can appear subset of php
						inside: {
							'comment': comment,
							'string': string,
							'attribute-class-name': [
								{
									pattern: /([^:]|^)\b[a-z_]\w*(?!\\)\b/i,
									alias: 'class-name',
									greedy: true,
									lookbehind: true
								},
								{
									pattern: /([^:]|^)(?:\\?\b[a-z_]\w*)+/i,
									alias: [
										'class-name',
										'class-name-fully-qualified'
									],
									greedy: true,
									lookbehind: true,
									inside: {
										'punctuation': /\\/
									}
								}
							],
							'constant': constant,
							'number': number,
							'operator': operator,
							'punctuation': punctuation
						}
					},
					'delimiter': {
						pattern: /^#\[|]$/,
						alias: 'punctuation'
					}
				}
			},
		});

		Prism.hooks.add('before-tokenize', function (env) {
			if (!/<\?/.test(env.code)) {
				return;
			}

			var phpPattern = /<\?(?:[^"'/#]|\/(?![*/])|("|')(?:\\[\s\S]|(?!\1)[^\\])*\1|(?:\/\/|#(?!\[))(?:[^?\n\r]|\?(?!>))*(?=$|\?>|[\r\n])|#\[|\/\*(?:[^*]|\*(?!\/))*(?:\*\/|$))*?(?:\?>|$)/ig;
			Prism.languages['markup-templating'].buildPlaceholders(env, 'php', phpPattern);
		});

		Prism.hooks.add('after-tokenize', function (env) {
			Prism.languages['markup-templating'].tokenizePlaceholders(env, 'php');
		});

	}(Prism));

	(function (Prism) {

		var javaDocLike = Prism.languages.javadoclike = {
			'parameter': {
				pattern: /(^\s*(?:\/{3}|\*|\/\*\*)\s*@(?:param|arg|arguments)\s+)\w+/m,
				lookbehind: true
			},
			'keyword': {
				// keywords are the first word in a line preceded be an `@` or surrounded by curly braces.
				// @word, {@word}
				pattern: /(^\s*(?:\/{3}|\*|\/\*\*)\s*|\{)@[a-z][a-zA-Z-]+\b/m,
				lookbehind: true
			},
			'punctuation': /[{}]/
		};


		/**
		 * Adds doc comment support to the given language and calls a given callback on each doc comment pattern.
		 *
		 * @param {string} lang the language add doc comment support to.
		 * @param {(pattern: {inside: {rest: undefined}}) => void} callback the function called with each doc comment pattern as argument.
		 */
		function docCommentSupport(lang, callback) {
			var tokenName = 'doc-comment';

			var grammar = Prism.languages[lang];
			if (!grammar) {
				return;
			}
			var token = grammar[tokenName];

			if (!token) {
				// add doc comment: /** */
				var definition = {};
				definition[tokenName] = {
					pattern: /(^|[^\\])\/\*\*[^/][\s\S]*?(?:\*\/|$)/,
					lookbehind: true,
					alias: 'comment'
				};

				grammar = Prism.languages.insertBefore(lang, 'comment', definition);
				token = grammar[tokenName];
			}

			if (token instanceof RegExp) { // convert regex to object
				token = grammar[tokenName] = { pattern: token };
			}

			if (Array.isArray(token)) {
				for (var i = 0, l = token.length; i < l; i++) {
					if (token[i] instanceof RegExp) {
						token[i] = { pattern: token[i] };
					}
					callback(token[i]);
				}
			} else {
				callback(token);
			}
		}

		/**
		 * Adds doc-comment support to the given languages for the given documentation language.
		 *
		 * @param {string[]|string} languages
		 * @param {Object} docLanguage
		 */
		function addSupport(languages, docLanguage) {
			if (typeof languages === 'string') {
				languages = [languages];
			}

			languages.forEach(function (lang) {
				docCommentSupport(lang, function (pattern) {
					if (!pattern.inside) {
						pattern.inside = {};
					}
					pattern.inside.rest = docLanguage;
				});
			});
		}

		Object.defineProperty(javaDocLike, 'addSupport', { value: addSupport });

		javaDocLike.addSupport(['java', 'javascript', 'php'], javaDocLike);

	}(Prism));

	(function (Prism) {

		var codeLinePattern = /(^(?:\s*(?:\*\s*)*))[^*\s].*$/m;

		var memberReference = /#\s*\w+(?:\s*\([^()]*\))?/.source;
		var reference = /(?:[a-zA-Z]\w+\s*\.\s*)*[A-Z]\w*(?:\s*<mem>)?|<mem>/.source.replace(/<mem>/g, function () { return memberReference; });

		Prism.languages.javadoc = Prism.languages.extend('javadoclike', {});
		Prism.languages.insertBefore('javadoc', 'keyword', {
			'reference': {
				pattern: RegExp(/(@(?:exception|throws|see|link|linkplain|value)\s+(?:\*\s*)?)/.source + '(?:' + reference + ')'),
				lookbehind: true,
				inside: {
					'function': {
						pattern: /(#\s*)\w+(?=\s*\()/,
						lookbehind: true
					},
					'field': {
						pattern: /(#\s*)\w+/,
						lookbehind: true
					},
					'namespace': {
						pattern: /\b(?:[a-z]\w*\s*\.\s*)+/,
						inside: {
							'punctuation': /\./
						}
					},
					'class-name': /\b[A-Z]\w*/,
					'keyword': Prism.languages.java.keyword,
					'punctuation': /[#()[\],.]/
				}
			},
			'class-name': {
				// @param <T> the first generic type parameter
				pattern: /(@param\s+)<[A-Z]\w*>/,
				lookbehind: true,
				inside: {
					'punctuation': /[.<>]/
				}
			},
			'code-section': [
				{
					pattern: /(\{@code\s+(?!\s))(?:[^\s{}]|\s+(?![\s}])|\{(?:[^{}]|\{(?:[^{}]|\{(?:[^{}]|\{[^{}]*\})*\})*\})*\})+(?=\s*\})/,
					lookbehind: true,
					inside: {
						'code': {
							// there can't be any HTML inside of {@code} tags
							pattern: codeLinePattern,
							lookbehind: true,
							inside: Prism.languages.java,
							alias: 'language-java'
						}
					}
				},
				{
					pattern: /(<(code|pre|tt)>(?!<code>)\s*)\S(?:\S|\s+\S)*?(?=\s*<\/\2>)/,
					lookbehind: true,
					inside: {
						'line': {
							pattern: codeLinePattern,
							lookbehind: true,
							inside: {
								// highlight HTML tags and entities
								'tag': Prism.languages.markup.tag,
								'entity': Prism.languages.markup.entity,
								'code': {
									// everything else is Java code
									pattern: /.+/,
									inside: Prism.languages.java,
									alias: 'language-java'
								}
							}
						}
					}
				}
			],
			'tag': Prism.languages.markup.tag,
			'entity': Prism.languages.markup.entity,
		});

		Prism.languages.javadoclike.addSupport('java', Prism.languages.javadoc);
	}(Prism));

	(function (Prism) {

		Prism.languages.typescript = Prism.languages.extend('javascript', {
			'class-name': {
				pattern: /(\b(?:class|extends|implements|instanceof|interface|new|type)\s+)(?!keyof\b)(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?:\s*<(?:[^<>]|<(?:[^<>]|<[^<>]*>)*>)*>)?/,
				lookbehind: true,
				greedy: true,
				inside: null // see below
			},
			'builtin': /\b(?:string|Function|any|number|boolean|Array|symbol|console|Promise|unknown|never)\b/,
		});

		// The keywords TypeScript adds to JavaScript
		Prism.languages.typescript.keyword.push(
			/\b(?:abstract|as|declare|implements|is|keyof|readonly|require)\b/,
			// keywords that have to be followed by an identifier
			/\b(?:asserts|infer|interface|module|namespace|type)(?!\s*[^\s_${}*a-zA-Z\xA0-\uFFFF])/
		);

		// doesn't work with TS because TS is too complex
		delete Prism.languages.typescript['parameter'];

		// a version of typescript specifically for highlighting types
		var typeInside = Prism.languages.extend('typescript', {});
		delete typeInside['class-name'];

		Prism.languages.typescript['class-name'].inside = typeInside;

		Prism.languages.insertBefore('typescript', 'function', {
			'decorator': {
				pattern: /@[$\w\xA0-\uFFFF]+/,
				inside: {
					'at': {
						pattern: /^@/,
						alias: 'operator'
					},
					'function': /^[\s\S]+/
				}
			},
			'generic-function': {
				// e.g. foo<T extends "bar" | "baz">( ...
				pattern: /#?(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*\s*<(?:[^<>]|<(?:[^<>]|<[^<>]*>)*>)*>(?=\s*\()/,
				greedy: true,
				inside: {
					'function': /^#?(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*/,
					'generic': {
						pattern: /<[\s\S]+/, // everything after the first <
						alias: 'class-name',
						inside: typeInside
					}
				}
			}
		});

		Prism.languages.ts = Prism.languages.typescript;

	}(Prism));

	(function (Prism) {

		var javascript = Prism.languages.javascript;

		var type = /{(?:[^{}]|{(?:[^{}]|{[^{}]*})*})+}/.source;
		var parameterPrefix = '(@(?:param|arg|argument|property)\\s+(?:' + type + '\\s+)?)';

		Prism.languages.jsdoc = Prism.languages.extend('javadoclike', {
			'parameter': {
				// @param {string} foo - foo bar
				pattern: RegExp(parameterPrefix + /(?:(?!\s)[$\w\xA0-\uFFFF.])+(?=\s|$)/.source),
				lookbehind: true,
				inside: {
					'punctuation': /\./
				}
			}
		});

		Prism.languages.insertBefore('jsdoc', 'keyword', {
			'optional-parameter': {
				// @param {string} [baz.foo="bar"] foo bar
				pattern: RegExp(parameterPrefix + /\[(?:(?!\s)[$\w\xA0-\uFFFF.])+(?:=[^[\]]+)?\](?=\s|$)/.source),
				lookbehind: true,
				inside: {
					'parameter': {
						pattern: /(^\[)[$\w\xA0-\uFFFF\.]+/,
						lookbehind: true,
						inside: {
							'punctuation': /\./
						}
					},
					'code': {
						pattern: /(=)[\s\S]*(?=\]$)/,
						lookbehind: true,
						inside: javascript,
						alias: 'language-javascript'
					},
					'punctuation': /[=[\]]/
				}
			},
			'class-name': [
				{
					pattern: RegExp(/(@(?:augments|extends|class|interface|memberof!?|template|this|typedef)\s+(?:<TYPE>\s+)?)[A-Z]\w*(?:\.[A-Z]\w*)*/.source.replace(/<TYPE>/g, function () { return type; })),
					lookbehind: true,
					inside: {
						'punctuation': /\./
					}
				},
				{
					pattern: RegExp('(@[a-z]+\\s+)' + type),
					lookbehind: true,
					inside: {
						'string': javascript.string,
						'number': javascript.number,
						'boolean': javascript.boolean,
						'keyword': Prism.languages.typescript.keyword,
						'operator': /=>|\.\.\.|[&|?:*]/,
						'punctuation': /[.,;=<>{}()[\]]/
					}
				}
			],
			'example': {
				pattern: /(@example\s+(?!\s))(?:[^@\s]|\s+(?!\s))+?(?=\s*(?:\*\s*)?(?:@\w|\*\/))/,
				lookbehind: true,
				inside: {
					'code': {
						pattern: /^(\s*(?:\*\s*)?)\S.*$/m,
						lookbehind: true,
						inside: javascript,
						alias: 'language-javascript'
					}
				}
			}
		});

		Prism.languages.javadoclike.addSupport('javascript', Prism.languages.jsdoc);

	}(Prism));

	(function (Prism) {

		Prism.languages.insertBefore('javascript', 'function-variable', {
			'method-variable': {
				pattern: RegExp('(\\.\\s*)' + Prism.languages.javascript['function-variable'].pattern.source),
				lookbehind: true,
				alias: ['function-variable', 'method', 'function', 'property-access']
			}
		});

		Prism.languages.insertBefore('javascript', 'function', {
			'method': {
				pattern: RegExp('(\\.\\s*)' + Prism.languages.javascript['function'].source),
				lookbehind: true,
				alias: ['function', 'property-access']
			}
		});

		Prism.languages.insertBefore('javascript', 'constant', {
			'known-class-name': [
				{
					// standard built-ins
					// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects
					pattern: /\b(?:(?:(?:Uint|Int)(?:8|16|32)|Uint8Clamped|Float(?:32|64))?Array|ArrayBuffer|BigInt|Boolean|DataView|Date|Error|Function|Intl|JSON|Math|Number|Object|Promise|Proxy|Reflect|RegExp|String|Symbol|(?:Weak)?(?:Set|Map)|WebAssembly)\b/,
					alias: 'class-name'
				},
				{
					// errors
					pattern: /\b(?:[A-Z]\w*)Error\b/,
					alias: 'class-name'
				}
			]
		});

		/**
		 * Replaces the `<ID>` placeholder in the given pattern with a pattern for general JS identifiers.
		 *
		 * @param {string} source
		 * @param {string} [flags]
		 * @returns {RegExp}
		 */
		function withId(source, flags) {
			return RegExp(
				source.replace(/<ID>/g, function () { return /(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*/.source; }),
				flags);
		}
		Prism.languages.insertBefore('javascript', 'keyword', {
			'imports': {
				// https://tc39.es/ecma262/#sec-imports
				pattern: withId(/(\bimport\b\s*)(?:<ID>(?:\s*,\s*(?:\*\s*as\s+<ID>|\{[^{}]*\}))?|\*\s*as\s+<ID>|\{[^{}]*\})(?=\s*\bfrom\b)/.source),
				lookbehind: true,
				inside: Prism.languages.javascript
			},
			'exports': {
				// https://tc39.es/ecma262/#sec-exports
				pattern: withId(/(\bexport\b\s*)(?:\*(?:\s*as\s+<ID>)?(?=\s*\bfrom\b)|\{[^{}]*\})/.source),
				lookbehind: true,
				inside: Prism.languages.javascript
			}
		});

		Prism.languages.javascript['keyword'].unshift(
			{
				pattern: /\b(?:as|default|export|from|import)\b/,
				alias: 'module'
			},
			{
				pattern: /\b(?:await|break|catch|continue|do|else|for|finally|if|return|switch|throw|try|while|yield)\b/,
				alias: 'control-flow'
			},
			{
				pattern: /\bnull\b/,
				alias: ['null', 'nil']
			},
			{
				pattern: /\bundefined\b/,
				alias: 'nil'
			}
		);

		Prism.languages.insertBefore('javascript', 'operator', {
			'spread': {
				pattern: /\.{3}/,
				alias: 'operator'
			},
			'arrow': {
				pattern: /=>/,
				alias: 'operator'
			}
		});

		Prism.languages.insertBefore('javascript', 'punctuation', {
			'property-access': {
				pattern: withId(/(\.\s*)#?<ID>/.source),
				lookbehind: true
			},
			'maybe-class-name': {
				pattern: /(^|[^$\w\xA0-\uFFFF])[A-Z][$\w\xA0-\uFFFF]+/,
				lookbehind: true
			},
			'dom': {
				// this contains only a few commonly used DOM variables
				pattern: /\b(?:document|location|navigator|performance|(?:local|session)Storage|window)\b/,
				alias: 'variable'
			},
			'console': {
				pattern: /\bconsole(?=\s*\.)/,
				alias: 'class-name'
			}
		});


		// add 'maybe-class-name' to tokens which might be a class name
		var maybeClassNameTokens = ['function', 'function-variable', 'method', 'method-variable', 'property-access'];

		for (var i = 0; i < maybeClassNameTokens.length; i++) {
			var token = maybeClassNameTokens[i];
			var value = Prism.languages.javascript[token];

			// convert regex to object
			if (Prism.util.type(value) === 'RegExp') {
				value = Prism.languages.javascript[token] = {
					pattern: value
				};
			}

			// keep in mind that we don't support arrays

			var inside = value.inside || {};
			value.inside = inside;

			inside['maybe-class-name'] = /^[A-Z][\s\S]*/;
		}

	}(Prism));

	// https://www.json.org/json-en.html
	Prism.languages.json = {
		'property': {
			pattern: /(^|[^\\])"(?:\\.|[^\\"\r\n])*"(?=\s*:)/,
			lookbehind: true,
			greedy: true
		},
		'string': {
			pattern: /(^|[^\\])"(?:\\.|[^\\"\r\n])*"(?!\s*:)/,
			lookbehind: true,
			greedy: true
		},
		'comment': {
			pattern: /\/\/.*|\/\*[\s\S]*?(?:\*\/|$)/,
			greedy: true
		},
		'number': /-?\b\d+(?:\.\d+)?(?:e[+-]?\d+)?\b/i,
		'punctuation': /[{}[\],]/,
		'operator': /:/,
		'boolean': /\b(?:true|false)\b/,
		'null': {
			pattern: /\bnull\b/,
			alias: 'keyword'
		}
	};

	Prism.languages.webmanifest = Prism.languages.json;

	(function (Prism) {

		var string = /("|')(?:\\(?:\r\n?|\n|.)|(?!\1)[^\\\r\n])*\1/;

		Prism.languages.json5 = Prism.languages.extend('json', {
			'property': [
				{
					pattern: RegExp(string.source + '(?=\\s*:)'),
					greedy: true
				},
				{
					pattern: /(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\s*:)/,
					alias: 'unquoted'
				}
			],
			'string': {
				pattern: string,
				greedy: true
			},
			'number': /[+-]?\b(?:NaN|Infinity|0x[a-fA-F\d]+)\b|[+-]?(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[eE][+-]?\d+\b)?/
		});

	}(Prism));

	Prism.languages.jsonp = Prism.languages.extend('json', {
		'punctuation': /[{}[\]();,.]/
	});

	Prism.languages.insertBefore('jsonp', 'punctuation', {
		'function': /(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\s*\()/
	});

	Prism.languages.jsstacktrace = {
		'error-message': {
			pattern: /^\S.*/m,
			alias: 'string'
		},

		'stack-frame': {
			pattern: /^[ \t]+at[ \t].*/m,
			inside: {
				'not-my-code': {
					pattern: /[ \t]+at[ \t]+(?!\s)(?:node\.js|\<unknown\>|.*(?:node_modules|\(\<anonymous\>\)|\(\<unknown\>|\<anonymous\>$|\(internal\/|\(node\.js)).*/m,
					alias: 'comment'
				},

				'filename': {
					pattern: /(\bat\s+(?!\s)|\()(?:[a-zA-Z]:)?[^():]+(?=:)/,
					lookbehind: true,
					alias: 'url'
				},

				'function': {
					pattern: /(at\s+(?:new\s+)?)(?!\s)[_$a-zA-Z\xA0-\uFFFF<][.$\w\xA0-\uFFFF<>]*/,
					lookbehind: true,
					inside: {
						'punctuation': /\./
					}
				},

				'punctuation': /[()]/,

				'keyword': /\b(?:at|new)\b/,

				'alias': {
					pattern: /\[(?:as\s+)?(?!\s)[_$a-zA-Z\xA0-\uFFFF][$\w\xA0-\uFFFF]*\]/,
					alias: 'variable'
				},

				'line-number': {
					pattern: /:[0-9]+(?::[0-9]+)?\b/,
					alias: 'number',
					inside: {
						'punctuation': /:/
					}
				},

			}
		}
	};

	Prism.languages.julia = {
		'comment': {
			// support one level of nested comments
			// https://github.com/JuliaLang/julia/pull/6128
			pattern: /(^|[^\\])(?:#=(?:[^#=]|=(?!#)|#(?!=)|#=(?:[^#=]|=(?!#)|#(?!=))*=#)*=#|#.*)/,
			lookbehind: true
		},
		'regex': {
			// https://docs.julialang.org/en/v1/manual/strings/#Regular-Expressions-1
			pattern: /r"(?:\\.|[^"\\\r\n])*"[imsx]{0,4}/,
			greedy: true
		},
		'string': {
			// https://docs.julialang.org/en/v1/manual/strings/#man-characters-1
			// https://docs.julialang.org/en/v1/manual/strings/#String-Basics-1
			// https://docs.julialang.org/en/v1/manual/strings/#non-standard-string-literals-1
			// https://docs.julialang.org/en/v1/manual/running-external-programs/#Running-External-Programs-1
			pattern: /"""[\s\S]+?"""|\w*"(?:\\.|[^"\\\r\n])*"|(^|[^\w'])'(?:\\[^\r\n][^'\r\n]*|[^\\\r\n])'|`(?:[^\\`\r\n]|\\.)*`/,
			lookbehind: true,
			greedy: true
		},
		'keyword': /\b(?:abstract|baremodule|begin|bitstype|break|catch|ccall|const|continue|do|else|elseif|end|export|finally|for|function|global|if|immutable|import|importall|in|let|local|macro|module|print|println|quote|return|struct|try|type|typealias|using|while)\b/,
		'boolean': /\b(?:true|false)\b/,
		'number': /(?:\b(?=\d)|\B(?=\.))(?:0[box])?(?:[\da-f]+(?:_[\da-f]+)*(?:\.(?:\d+(?:_\d+)*)?)?|\.\d+(?:_\d+)*)(?:[efp][+-]?\d+(?:_\d+)*)?j?/i,
		// https://docs.julialang.org/en/v1/manual/mathematical-operations/
		// https://docs.julialang.org/en/v1/manual/mathematical-operations/#Operator-Precedence-and-Associativity-1
		'operator': /&&|\|\||[-+*^%÷⊻&$\\]=?|\/[\/=]?|!=?=?|\|[=>]?|<(?:<=?|[=:|])?|>(?:=|>>?=?)?|==?=?|[~≠≤≥'√∛]/,
		'punctuation': /::?|[{}[\]();,.?]/,
		// https://docs.julialang.org/en/v1/base/numbers/#Base.im
		'constant': /\b(?:(?:NaN|Inf)(?:16|32|64)?|im|pi)\b|[πℯ]/
	};

	(function (Prism) {
		Prism.languages.kotlin = Prism.languages.extend('clike', {
			'keyword': {
				// The lookbehind prevents wrong highlighting of e.g. kotlin.properties.get
				pattern: /(^|[^.])\b(?:abstract|actual|annotation|as|break|by|catch|class|companion|const|constructor|continue|crossinline|data|do|dynamic|else|enum|expect|external|final|finally|for|fun|get|if|import|in|infix|init|inline|inner|interface|internal|is|lateinit|noinline|null|object|open|operator|out|override|package|private|protected|public|reified|return|sealed|set|super|suspend|tailrec|this|throw|to|try|typealias|val|var|vararg|when|where|while)\b/,
				lookbehind: true
			},
			'function': [
				{
					pattern: /(?:`[^\r\n`]+`|\w+)(?=\s*\()/,
					greedy: true
				},
				{
					pattern: /(\.)(?:`[^\r\n`]+`|\w+)(?=\s*\{)/,
					lookbehind: true,
					greedy: true
				}
			],
			'number': /\b(?:0[xX][\da-fA-F]+(?:_[\da-fA-F]+)*|0[bB][01]+(?:_[01]+)*|\d+(?:_\d+)*(?:\.\d+(?:_\d+)*)?(?:[eE][+-]?\d+(?:_\d+)*)?[fFL]?)\b/,
			'operator': /\+[+=]?|-[-=>]?|==?=?|!(?:!|==?)?|[\/*%<>]=?|[?:]:?|\.\.|&&|\|\||\b(?:and|inv|or|shl|shr|ushr|xor)\b/
		});

		delete Prism.languages.kotlin['class-name'];

		Prism.languages.insertBefore('kotlin', 'string', {
			'raw-string': {
				pattern: /("""|''')[\s\S]*?\1/,
				alias: 'string'
				// See interpolation below
			}
		});
		Prism.languages.insertBefore('kotlin', 'keyword', {
			'annotation': {
				pattern: /\B@(?:\w+:)?(?:[A-Z]\w*|\[[^\]]+\])/,
				alias: 'builtin'
			}
		});
		Prism.languages.insertBefore('kotlin', 'function', {
			'label': {
				pattern: /\w+@|@\w+/,
				alias: 'symbol'
			}
		});

		var interpolation = [
			{
				pattern: /\$\{[^}]+\}/,
				inside: {
					'delimiter': {
						pattern: /^\$\{|\}$/,
						alias: 'variable'
					},
					rest: Prism.languages.kotlin
				}
			},
			{
				pattern: /\$\w+/,
				alias: 'variable'
			}
		];

		Prism.languages.kotlin['string'].inside = Prism.languages.kotlin['raw-string'].inside = {
			interpolation: interpolation
		};

		Prism.languages.kt = Prism.languages.kotlin;
		Prism.languages.kts = Prism.languages.kotlin;
	}(Prism));

	(function (Prism) {
		var funcPattern = /\\(?:[^a-z()[\]]|[a-z*]+)/i;
		var insideEqu = {
			'equation-command': {
				pattern: funcPattern,
				alias: 'regex'
			}
		};

		Prism.languages.latex = {
			'comment': /%.*/m,
			// the verbatim environment prints whitespace to the document
			'cdata': {
				pattern: /(\\begin\{((?:verbatim|lstlisting)\*?)\})[\s\S]*?(?=\\end\{\2\})/,
				lookbehind: true
			},
			/*
			 * equations can be between $$ $$ or $ $ or \( \) or \[ \]
			 * (all are multiline)
			 */
			'equation': [
				{
					pattern: /\$\$(?:\\[\s\S]|[^\\$])+\$\$|\$(?:\\[\s\S]|[^\\$])+\$|\\\([\s\S]*?\\\)|\\\[[\s\S]*?\\\]/,
					inside: insideEqu,
					alias: 'string'
				},
				{
					pattern: /(\\begin\{((?:equation|math|eqnarray|align|multline|gather)\*?)\})[\s\S]*?(?=\\end\{\2\})/,
					lookbehind: true,
					inside: insideEqu,
					alias: 'string'
				}
			],
			/*
			 * arguments which are keywords or references are highlighted
			 * as keywords
			 */
			'keyword': {
				pattern: /(\\(?:begin|end|ref|cite|label|usepackage|documentclass)(?:\[[^\]]+\])?\{)[^}]+(?=\})/,
				lookbehind: true
			},
			'url': {
				pattern: /(\\url\{)[^}]+(?=\})/,
				lookbehind: true
			},
			/*
			 * section or chapter headlines are highlighted as bold so that
			 * they stand out more
			 */
			'headline': {
				pattern: /(\\(?:part|chapter|section|subsection|frametitle|subsubsection|paragraph|subparagraph|subsubparagraph|subsubsubparagraph)\*?(?:\[[^\]]+\])?\{)[^}]+(?=\}(?:\[[^\]]+\])?)/,
				lookbehind: true,
				alias: 'class-name'
			},
			'function': {
				pattern: funcPattern,
				alias: 'selector'
			},
			'punctuation': /[[\]{}&]/
		};

		Prism.languages.tex = Prism.languages.latex;
		Prism.languages.context = Prism.languages.latex;
	}(Prism));

	/* FIXME :
	 :extend() is not handled specifically : its highlighting is buggy.
	 Mixin usage must be inside a ruleset to be highlighted.
	 At-rules (e.g. import) containing interpolations are buggy.
	 Detached rulesets are highlighted as at-rules.
	 A comment before a mixin usage prevents the latter to be properly highlighted.
	 */

	Prism.languages.less = Prism.languages.extend('css', {
		'comment': [
			/\/\*[\s\S]*?\*\//,
			{
				pattern: /(^|[^\\])\/\/.*/,
				lookbehind: true
			}
		],
		'atrule': {
			pattern: /@[\w-](?:\((?:[^(){}]|\([^(){}]*\))*\)|[^(){};\s]|\s+(?!\s))*?(?=\s*\{)/,
			inside: {
				'punctuation': /[:()]/
			}
		},
		// selectors and mixins are considered the same
		'selector': {
			pattern: /(?:@\{[\w-]+\}|[^{};\s@])(?:@\{[\w-]+\}|\((?:[^(){}]|\([^(){}]*\))*\)|[^(){};@\s]|\s+(?!\s))*?(?=\s*\{)/,
			inside: {
				// mixin parameters
				'variable': /@+[\w-]+/
			}
		},

		'property': /(?:@\{[\w-]+\}|[\w-])+(?:\+_?)?(?=\s*:)/i,
		'operator': /[+\-*\/]/
	});

	Prism.languages.insertBefore('less', 'property', {
		'variable': [
			// Variable declaration (the colon must be consumed!)
			{
				pattern: /@[\w-]+\s*:/,
				inside: {
					'punctuation': /:/
				}
			},

			// Variable usage
			/@@?[\w-]+/
		],
		'mixin-usage': {
			pattern: /([{;]\s*)[.#](?!\d)[\w-].*?(?=[(;])/,
			lookbehind: true,
			alias: 'function'
		}
	});

	(function (Prism) {
		// Functions to construct regular expressions
		// simple form
		// e.g. (interactive ... or (interactive)
		function simple_form(name) {
			return RegExp('(\\()' + name + '(?=[\\s\\)])');
		}
		// booleans and numbers
		function primitive(pattern) {
			return RegExp('([\\s([])' + pattern + '(?=[\\s)])');
		}

		// Patterns in regular expressions

		// Symbol name. See https://www.gnu.org/software/emacs/manual/html_node/elisp/Symbol-Type.html
		// & and : are excluded as they are usually used for special purposes
		var symbol = '[-+*/_~!@$%^=<>{}\\w]+';
		// symbol starting with & used in function arguments
		var marker = '&' + symbol;
		// Open parenthesis for look-behind
		var par = '(\\()';
		var endpar = '(?=\\))';
		// End the pattern with look-ahead space
		var space = '(?=\\s)';

		var language = {
			// Three or four semicolons are considered a heading.
			// See https://www.gnu.org/software/emacs/manual/html_node/elisp/Comment-Tips.html
			heading: {
				pattern: /;;;.*/,
				alias: ['comment', 'title']
			},
			comment: /;.*/,
			string: {
				pattern: /"(?:[^"\\]|\\.)*"/,
				greedy: true,
				inside: {
					argument: /[-A-Z]+(?=[.,\s])/,
					symbol: RegExp('`' + symbol + "'")
				}
			},
			'quoted-symbol': {
				pattern: RegExp("#?'" + symbol),
				alias: ['variable', 'symbol']
			},
			'lisp-property': {
				pattern: RegExp(':' + symbol),
				alias: 'property'
			},
			splice: {
				pattern: RegExp(',@?' + symbol),
				alias: ['symbol', 'variable']
			},
			keyword: [
				{
					pattern: RegExp(
						par +
							'(?:(?:lexical-)?let\\*?|(?:cl-)?letf|if|when|while|unless|cons|cl-loop|and|or|not|cond|setq|error|message|null|require|provide|use-package)' +
							space
					),
					lookbehind: true
				},
				{
					pattern: RegExp(
						par + '(?:for|do|collect|return|finally|append|concat|in|by)' + space
					),
					lookbehind: true
				},
			],
			declare: {
				pattern: simple_form('declare'),
				lookbehind: true,
				alias: 'keyword'
			},
			interactive: {
				pattern: simple_form('interactive'),
				lookbehind: true,
				alias: 'keyword'
			},
			boolean: {
				pattern: primitive('(?:t|nil)'),
				lookbehind: true
			},
			number: {
				pattern: primitive('[-+]?\\d+(?:\\.\\d*)?'),
				lookbehind: true
			},
			defvar: {
				pattern: RegExp(par + 'def(?:var|const|custom|group)\\s+' + symbol),
				lookbehind: true,
				inside: {
					keyword: /^def[a-z]+/,
					variable: RegExp(symbol)
				}
			},
			defun: {
				pattern: RegExp(
					par +
						'(?:cl-)?(?:defun\\*?|defmacro)\\s+' +
						symbol +
						'\\s+\\([\\s\\S]*?\\)'
				),
				lookbehind: true,
				inside: {
					keyword: /^(?:cl-)?def\S+/,
					// See below, this property needs to be defined later so that it can
					// reference the language object.
					arguments: null,
					function: {
						pattern: RegExp('(^\\s)' + symbol),
						lookbehind: true
					},
					punctuation: /[()]/
				}
			},
			lambda: {
				pattern: RegExp(par + 'lambda\\s+\\(\\s*(?:&?' + symbol + '(?:\\s+&?' + symbol + ')*\\s*)?\\)'),
				lookbehind: true,
				inside: {
					keyword: /^lambda/,
					// See below, this property needs to be defined later so that it can
					// reference the language object.
					arguments: null,
					punctuation: /[()]/
				}
			},
			car: {
				pattern: RegExp(par + symbol),
				lookbehind: true
			},
			punctuation: [
				// open paren, brackets, and close paren
				/(?:['`,]?\(|[)\[\]])/,
				// cons
				{
					pattern: /(\s)\.(?=\s)/,
					lookbehind: true
				},
			]
		};

		var arg = {
			'lisp-marker': RegExp(marker),
			rest: {
				argument: {
					pattern: RegExp(symbol),
					alias: 'variable'
				},
				varform: {
					pattern: RegExp(par + symbol + '\\s+\\S[\\s\\S]*' + endpar),
					lookbehind: true,
					inside: {
						string: language.string,
						boolean: language.boolean,
						number: language.number,
						symbol: language.symbol,
						punctuation: /[()]/
					}
				}
			}
		};

		var forms = '\\S+(?:\\s+\\S+)*';

		var arglist = {
			pattern: RegExp(par + '[\\s\\S]*' + endpar),
			lookbehind: true,
			inside: {
				'rest-vars': {
					pattern: RegExp('&(?:rest|body)\\s+' + forms),
					inside: arg
				},
				'other-marker-vars': {
					pattern: RegExp('&(?:optional|aux)\\s+' + forms),
					inside: arg
				},
				keys: {
					pattern: RegExp('&key\\s+' + forms + '(?:\\s+&allow-other-keys)?'),
					inside: arg
				},
				argument: {
					pattern: RegExp(symbol),
					alias: 'variable'
				},
				punctuation: /[()]/
			}
		};

		language['lambda'].inside.arguments = arglist;
		language['defun'].inside.arguments = Prism.util.clone(arglist);
		language['defun'].inside.arguments.inside.sublist = arglist;

		Prism.languages.lisp = language;
		Prism.languages.elisp = language;
		Prism.languages.emacs = language;
		Prism.languages['emacs-lisp'] = language;
	}(Prism));

	// This is a language definition for generic log files.
	// Since there is no one log format, this language definition has to support all formats to some degree.
	//
	// Based on https://github.com/MTDL9/vim-log-highlighting

	Prism.languages.log = {
		'string': {
			// Single-quoted strings must not be confused with plain text. E.g. Can't isn't Susan's Chris' toy
			pattern: /"(?:[^"\\\r\n]|\\.)*"|'(?![st] | \w)(?:[^'\\\r\n]|\\.)*'/,
			greedy: true,
		},

		'level': [
			{
				pattern: /\b(?:ALERT|CRIT|CRITICAL|EMERG|EMERGENCY|ERR|ERROR|FAILURE|FATAL|SEVERE)\b/,
				alias: ['error', 'important']
			},
			{
				pattern: /\b(?:WARN|WARNING|WRN)\b/,
				alias: ['warning', 'important']
			},
			{
				pattern: /\b(?:DISPLAY|INF|INFO|NOTICE|STATUS)\b/,
				alias: ['info', 'keyword']
			},
			{
				pattern: /\b(?:DBG|DEBUG|FINE)\b/,
				alias: ['debug', 'keyword']
			},
			{
				pattern: /\b(?:FINER|FINEST|TRACE|TRC|VERBOSE|VRB)\b/,
				alias: ['trace', 'comment']
			}
		],

		'property': {
			pattern: /((?:^|[\]|])[ \t]*)[a-z_](?:[\w-]|\b\/\b)*(?:[. ]\(?\w(?:[\w-]|\b\/\b)*\)?)*:(?=\s)/im,
			lookbehind: true
		},

		'separator': {
			pattern: /(^|[^-+])-{3,}|={3,}|\*{3,}|- - /m,
			lookbehind: true,
			alias: 'comment'
		},

		'url': /\b(?:https?|ftp|file):\/\/[^\s|,;'"]*[^\s|,;'">.]/,
		'email': {
			pattern: /(^|\s)[-\w+.]+@[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)+(?=\s)/,
			lookbehind: true,
			alias: 'url'
		},

		'ip-address': {
			pattern: /\b(?:\d{1,3}(?:\.\d{1,3}){3})\b/i,
			alias: 'constant'
		},
		'mac-address': {
			pattern: /\b[a-f0-9]{2}(?::[a-f0-9]{2}){5}\b/i,
			alias: 'constant'
		},
		'domain': {
			pattern: /(^|\s)[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)*\.[a-z][a-z0-9-]+(?=\s)/,
			lookbehind: true,
			alias: 'constant'
		},

		'uuid': {
			pattern: /\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/i,
			alias: 'constant'
		},
		'hash': {
			pattern: /\b(?:[a-f0-9]{32}){1,2}\b/i,
			alias: 'constant'
		},

		'file-path': {
			pattern: /\b[a-z]:[\\/][^\s|,;:(){}\[\]"']+|(^|[\s:\[\](>|])\.{0,2}\/\w[^\s|,;:(){}\[\]"']*/i,
			lookbehind: true,
			greedy: true,
			alias: 'string'
		},

		'date': {
			pattern: RegExp(
				/\b\d{4}[-/]\d{2}[-/]\d{2}(?:T(?=\d{1,2}:)|(?=\s\d{1,2}:))/.source +
				'|' +
				/\b\d{1,4}[-/ ](?:\d{1,2}|Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[-/ ]\d{2,4}T?\b/.source +
				'|' +
				/\b(?:(?:Mon|Tue|Wed|Thu|Fri|Sat|Sun)(?:\s{1,2}(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec))?|Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s{1,2}\d{1,2}\b/.source,
				'i'
			),
			alias: 'number'
		},
		'time': {
			pattern: /\b\d{1,2}:\d{1,2}:\d{1,2}(?:[.,:]\d+)?(?:\s?[+-]\d{2}:?\d{2}|Z)?\b/,
			alias: 'number'
		},

		'boolean': /\b(?:true|false|null)\b/i,
		'number': {
			pattern: /(^|[^.\w])(?:0x[a-f0-9]+|0o[0-7]+|0b[01]+|v?\d[\da-f]*(?:\.\d+)*(?:e[+-]?\d+)?[a-z]{0,3}\b)\b(?!\.\w)/i,
			lookbehind: true
		},

		'operator': /[;:?<=>~/@!$%&+\-|^(){}*#]/,
		'punctuation': /[\[\].,]/
	};

	Prism.languages.lua = {
		'comment': /^#!.+|--(?:\[(=*)\[[\s\S]*?\]\1\]|.*)/m,
		// \z may be used to skip the following space
		'string': {
			pattern: /(["'])(?:(?!\1)[^\\\r\n]|\\z(?:\r\n|\s)|\\(?:\r\n|[^z]))*\1|\[(=*)\[[\s\S]*?\]\2\]/,
			greedy: true
		},
		'number': /\b0x[a-f\d]+(?:\.[a-f\d]*)?(?:p[+-]?\d+)?\b|\b\d+(?:\.\B|(?:\.\d*)?(?:e[+-]?\d+)?\b)|\B\.\d+(?:e[+-]?\d+)?\b/i,
		'keyword': /\b(?:and|break|do|else|elseif|end|false|for|function|goto|if|in|local|nil|not|or|repeat|return|then|true|until|while)\b/,
		'function': /(?!\d)\w+(?=\s*(?:[({]))/,
		'operator': [
			/[-+*%^&|#]|\/\/?|<[<=]?|>[>=]?|[=~]=?/,
			{
				// Match ".." but don't break "..."
				pattern: /(^|[^.])\.\.(?!\.)/,
				lookbehind: true
			}
		],
		'punctuation': /[\[\](){},;]|\.+|:+/
	};

	Prism.languages.makefile = {
		'comment': {
			pattern: /(^|[^\\])#(?:\\(?:\r\n|[\s\S])|[^\\\r\n])*/,
			lookbehind: true
		},
		'string': {
			pattern: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/,
			greedy: true
		},

		// Built-in target names
		'builtin': /\.[A-Z][^:#=\s]+(?=\s*:(?!=))/,

		// Targets
		'symbol': {
			pattern: /^(?:[^:=\s]|[ \t]+(?![\s:]))+(?=\s*:(?!=))/m,
			inside: {
				'variable': /\$+(?:(?!\$)[^(){}:#=\s]+|(?=[({]))/
			}
		},
		'variable': /\$+(?:(?!\$)[^(){}:#=\s]+|\([@*%<^+?][DF]\)|(?=[({]))/,

		'keyword': [
			// Directives
			/-include\b|\b(?:define|else|endef|endif|export|ifn?def|ifn?eq|include|override|private|sinclude|undefine|unexport|vpath)\b/,
			// Functions
			{
				pattern: /(\()(?:addsuffix|abspath|and|basename|call|dir|error|eval|file|filter(?:-out)?|findstring|firstword|flavor|foreach|guile|if|info|join|lastword|load|notdir|or|origin|patsubst|realpath|shell|sort|strip|subst|suffix|value|warning|wildcard|word(?:s|list)?)(?=[ \t])/,
				lookbehind: true
			}
		],
		'operator': /(?:::|[?:+!])?=|[|@]/,
		'punctuation': /[:;(){}]/
	};

	(function (Prism) {

		// Allow only one line break
		var inner = /(?:\\.|[^\\\n\r]|(?:\n|\r\n?)(?!\n|\r\n?))/.source;

		/**
		 * This function is intended for the creation of the bold or italic pattern.
		 *
		 * This also adds a lookbehind group to the given pattern to ensure that the pattern is not backslash-escaped.
		 *
		 * _Note:_ Keep in mind that this adds a capturing group.
		 *
		 * @param {string} pattern
		 * @returns {RegExp}
		 */
		function createInline(pattern) {
			pattern = pattern.replace(/<inner>/g, function () { return inner; });
			return RegExp(/((?:^|[^\\])(?:\\{2})*)/.source + '(?:' + pattern + ')');
		}


		var tableCell = /(?:\\.|``(?:[^`\r\n]|`(?!`))+``|`[^`\r\n]+`|[^\\|\r\n`])+/.source;
		var tableRow = /\|?__(?:\|__)+\|?(?:(?:\n|\r\n?)|(?![\s\S]))/.source.replace(/__/g, function () { return tableCell; });
		var tableLine = /\|?[ \t]*:?-{3,}:?[ \t]*(?:\|[ \t]*:?-{3,}:?[ \t]*)+\|?(?:\n|\r\n?)/.source;


		Prism.languages.markdown = Prism.languages.extend('markup', {});
		Prism.languages.insertBefore('markdown', 'prolog', {
			'front-matter-block': {
				pattern: /(^(?:\s*[\r\n])?)---(?!.)[\s\S]*?[\r\n]---(?!.)/,
				lookbehind: true,
				greedy: true,
				inside: {
					'punctuation': /^---|---$/,
					'font-matter': {
						pattern: /\S+(?:\s+\S+)*/,
						alias: ['yaml', 'language-yaml'],
						inside: Prism.languages.yaml
					}
				}
			},
			'blockquote': {
				// > ...
				pattern: /^>(?:[\t ]*>)*/m,
				alias: 'punctuation'
			},
			'table': {
				pattern: RegExp('^' + tableRow + tableLine + '(?:' + tableRow + ')*', 'm'),
				inside: {
					'table-data-rows': {
						pattern: RegExp('^(' + tableRow + tableLine + ')(?:' + tableRow + ')*$'),
						lookbehind: true,
						inside: {
							'table-data': {
								pattern: RegExp(tableCell),
								inside: Prism.languages.markdown
							},
							'punctuation': /\|/
						}
					},
					'table-line': {
						pattern: RegExp('^(' + tableRow + ')' + tableLine + '$'),
						lookbehind: true,
						inside: {
							'punctuation': /\||:?-{3,}:?/
						}
					},
					'table-header-row': {
						pattern: RegExp('^' + tableRow + '$'),
						inside: {
							'table-header': {
								pattern: RegExp(tableCell),
								alias: 'important',
								inside: Prism.languages.markdown
							},
							'punctuation': /\|/
						}
					}
				}
			},
			'code': [
				{
					// Prefixed by 4 spaces or 1 tab and preceded by an empty line
					pattern: /((?:^|\n)[ \t]*\n|(?:^|\r\n?)[ \t]*\r\n?)(?: {4}|\t).+(?:(?:\n|\r\n?)(?: {4}|\t).+)*/,
					lookbehind: true,
					alias: 'keyword'
				},
				{
					// `code`
					// ``code``
					pattern: /``.+?``|`[^`\r\n]+`/,
					alias: 'keyword'
				},
				{
					// ```optional language
					// code block
					// ```
					pattern: /^```[\s\S]*?^```$/m,
					greedy: true,
					inside: {
						'code-block': {
							pattern: /^(```.*(?:\n|\r\n?))[\s\S]+?(?=(?:\n|\r\n?)^```$)/m,
							lookbehind: true
						},
						'code-language': {
							pattern: /^(```).+/,
							lookbehind: true
						},
						'punctuation': /```/
					}
				}
			],
			'title': [
				{
					// title 1
					// =======

					// title 2
					// -------
					pattern: /\S.*(?:\n|\r\n?)(?:==+|--+)(?=[ \t]*$)/m,
					alias: 'important',
					inside: {
						punctuation: /==+$|--+$/
					}
				},
				{
					// # title 1
					// ###### title 6
					pattern: /(^\s*)#.+/m,
					lookbehind: true,
					alias: 'important',
					inside: {
						punctuation: /^#+|#+$/
					}
				}
			],
			'hr': {
				// ***
				// ---
				// * * *
				// -----------
				pattern: /(^\s*)([*-])(?:[\t ]*\2){2,}(?=\s*$)/m,
				lookbehind: true,
				alias: 'punctuation'
			},
			'list': {
				// * item
				// + item
				// - item
				// 1. item
				pattern: /(^\s*)(?:[*+-]|\d+\.)(?=[\t ].)/m,
				lookbehind: true,
				alias: 'punctuation'
			},
			'url-reference': {
				// [id]: http://example.com "Optional title"
				// [id]: http://example.com 'Optional title'
				// [id]: http://example.com (Optional title)
				// [id]: <http://example.com> "Optional title"
				pattern: /!?\[[^\]]+\]:[\t ]+(?:\S+|<(?:\\.|[^>\\])+>)(?:[\t ]+(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\((?:\\.|[^)\\])*\)))?/,
				inside: {
					'variable': {
						pattern: /^(!?\[)[^\]]+/,
						lookbehind: true
					},
					'string': /(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\((?:\\.|[^)\\])*\))$/,
					'punctuation': /^[\[\]!:]|[<>]/
				},
				alias: 'url'
			},
			'bold': {
				// **strong**
				// __strong__

				// allow one nested instance of italic text using the same delimiter
				pattern: createInline(/\b__(?:(?!_)<inner>|_(?:(?!_)<inner>)+_)+__\b|\*\*(?:(?!\*)<inner>|\*(?:(?!\*)<inner>)+\*)+\*\*/.source),
				lookbehind: true,
				greedy: true,
				inside: {
					'content': {
						pattern: /(^..)[\s\S]+(?=..$)/,
						lookbehind: true,
						inside: {} // see below
					},
					'punctuation': /\*\*|__/
				}
			},
			'italic': {
				// *em*
				// _em_

				// allow one nested instance of bold text using the same delimiter
				pattern: createInline(/\b_(?:(?!_)<inner>|__(?:(?!_)<inner>)+__)+_\b|\*(?:(?!\*)<inner>|\*\*(?:(?!\*)<inner>)+\*\*)+\*/.source),
				lookbehind: true,
				greedy: true,
				inside: {
					'content': {
						pattern: /(^.)[\s\S]+(?=.$)/,
						lookbehind: true,
						inside: {} // see below
					},
					'punctuation': /[*_]/
				}
			},
			'strike': {
				// ~~strike through~~
				// ~strike~
				pattern: createInline(/(~~?)(?:(?!~)<inner>)+?\2/.source),
				lookbehind: true,
				greedy: true,
				inside: {
					'content': {
						pattern: /(^~~?)[\s\S]+(?=\1$)/,
						lookbehind: true,
						inside: {} // see below
					},
					'punctuation': /~~?/
				}
			},
			'url': {
				// [example](http://example.com "Optional title")
				// [example][id]
				// [example] [id]
				pattern: createInline(/!?\[(?:(?!\])<inner>)+\](?:\([^\s)]+(?:[\t ]+"(?:\\.|[^"\\])*")?\)|[ \t]?\[(?:(?!\])<inner>)+\])/.source),
				lookbehind: true,
				greedy: true,
				inside: {
					'operator': /^!/,
					'content': {
						pattern: /(^\[)[^\]]+(?=\])/,
						lookbehind: true,
						inside: {} // see below
					},
					'variable': {
						pattern: /(^\][ \t]?\[)[^\]]+(?=\]$)/,
						lookbehind: true
					},
					'url': {
						pattern: /(^\]\()[^\s)]+/,
						lookbehind: true
					},
					'string': {
						pattern: /(^[ \t]+)"(?:\\.|[^"\\])*"(?=\)$)/,
						lookbehind: true
					}
				}
			}
		});

		['url', 'bold', 'italic', 'strike'].forEach(function (token) {
			['url', 'bold', 'italic', 'strike'].forEach(function (inside) {
				if (token !== inside) {
					Prism.languages.markdown[token].inside.content.inside[inside] = Prism.languages.markdown[inside];
				}
			});
		});

		Prism.hooks.add('after-tokenize', function (env) {
			if (env.language !== 'markdown' && env.language !== 'md') {
				return;
			}

			function walkTokens(tokens) {
				if (!tokens || typeof tokens === 'string') {
					return;
				}

				for (var i = 0, l = tokens.length; i < l; i++) {
					var token = tokens[i];

					if (token.type !== 'code') {
						walkTokens(token.content);
						continue;
					}

					/*
					 * Add the correct `language-xxxx` class to this code block. Keep in mind that the `code-language` token
					 * is optional. But the grammar is defined so that there is only one case we have to handle:
					 *
					 * token.content = [
					 *     <span class="punctuation">```</span>,
					 *     <span class="code-language">xxxx</span>,
					 *     '\n', // exactly one new lines (\r or \n or \r\n)
					 *     <span class="code-block">...</span>,
					 *     '\n', // exactly one new lines again
					 *     <span class="punctuation">```</span>
					 * ];
					 */

					var codeLang = token.content[1];
					var codeBlock = token.content[3];

					if (codeLang && codeBlock &&
						codeLang.type === 'code-language' && codeBlock.type === 'code-block' &&
						typeof codeLang.content === 'string') {

						// this might be a language that Prism does not support

						// do some replacements to support C++, C#, and F#
						var lang = codeLang.content.replace(/\b#/g, 'sharp').replace(/\b\+\+/g, 'pp');
						// only use the first word
						lang = (/[a-z][\w-]*/i.exec(lang) || [''])[0].toLowerCase();
						var alias = 'language-' + lang;

						// add alias
						if (!codeBlock.alias) {
							codeBlock.alias = [alias];
						} else if (typeof codeBlock.alias === 'string') {
							codeBlock.alias = [codeBlock.alias, alias];
						} else {
							codeBlock.alias.push(alias);
						}
					}
				}
			}

			walkTokens(env.tokens);
		});

		Prism.hooks.add('wrap', function (env) {
			if (env.type !== 'code-block') {
				return;
			}

			var codeLang = '';
			for (var i = 0, l = env.classes.length; i < l; i++) {
				var cls = env.classes[i];
				var match = /language-(.+)/.exec(cls);
				if (match) {
					codeLang = match[1];
					break;
				}
			}

			var grammar = Prism.languages[codeLang];

			if (!grammar) {
				if (codeLang && codeLang !== 'none' && Prism.plugins.autoloader) {
					var id = 'md-' + new Date().valueOf() + '-' + Math.floor(Math.random() * 1e16);
					env.attributes['id'] = id;

					Prism.plugins.autoloader.loadLanguages(codeLang, function () {
						var ele = document.getElementById(id);
						if (ele) {
							ele.innerHTML = Prism.highlight(ele.textContent, Prism.languages[codeLang], codeLang);
						}
					});
				}
			} else {
				// get the textContent of the given env HTML
				var tempContainer = document.createElement('div');
				tempContainer.innerHTML = env.content;
				var code = tempContainer.textContent;

				env.content = Prism.highlight(code, grammar, codeLang);
			}
		});

		Prism.languages.md = Prism.languages.markdown;

	}(Prism));

	Prism.languages.matlab = {
		'comment': [
			/%\{[\s\S]*?\}%/,
			/%.+/
		],
		'string': {
			pattern: /\B'(?:''|[^'\r\n])*'/,
			greedy: true
		},
		// FIXME We could handle imaginary numbers as a whole
		'number': /(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[eE][+-]?\d+)?(?:[ij])?|\b[ij]\b/,
		'keyword': /\b(?:break|case|catch|continue|else|elseif|end|for|function|if|inf|NaN|otherwise|parfor|pause|pi|return|switch|try|while)\b/,
		'function': /(?!\d)\w+(?=\s*\()/,
		'operator': /\.?[*^\/\\']|[+\-:@]|[<>=~]=?|&&?|\|\|?/,
		'punctuation': /\.{3}|[.,;\[\](){}!]/
	};

	Prism.languages.nasm = {
		'comment': /;.*$/m,
		'string': /(["'`])(?:\\.|(?!\1)[^\\\r\n])*\1/,
		'label': {
			pattern: /(^\s*)[A-Za-z._?$][\w.?$@~#]*:/m,
			lookbehind: true,
			alias: 'function'
		},
		'keyword': [
			/\[?BITS (?:16|32|64)\]?/,
			{
				pattern: /(^\s*)section\s*[a-zA-Z.]+:?/im,
				lookbehind: true
			},
			/(?:extern|global)[^;\r\n]*/i,
			/(?:CPU|FLOAT|DEFAULT).*$/m
		],
		'register': {
			pattern: /\b(?:st\d|[xyz]mm\d\d?|[cdt]r\d|r\d\d?[bwd]?|[er]?[abcd]x|[abcd][hl]|[er]?(?:bp|sp|si|di)|[cdefgs]s)\b/i,
			alias: 'variable'
		},
		'number': /(?:\b|(?=\$))(?:0[hx](?:\.[\da-f]+|[\da-f]+(?:\.[\da-f]+)?)(?:p[+-]?\d+)?|\d[\da-f]+[hx]|\$\d[\da-f]*|0[oq][0-7]+|[0-7]+[oq]|0[by][01]+|[01]+[by]|0[dt]\d+|(?:\d+(?:\.\d+)?|\.\d+)(?:\.?e[+-]?\d+)?[dt]?)\b/i,
		'operator': /[\[\]*+\-\/%<>=&|$!]/
	};

	(function (Prism) {

		var variable = /\$(?:\w[a-z\d]*(?:_[^\x00-\x1F\s"'\\()$]*)?|\{[^}\s"'\\]+\})/i;

		Prism.languages.nginx = {
			'comment': {
				pattern: /(^|[\s{};])#.*/,
				lookbehind: true
			},
			'directive': {
				pattern: /(^|\s)\w(?:[^;{}"'\\\s]|\\.|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\s+(?:#.*(?!.)|(?![#\s])))*?(?=\s*[;{])/,
				lookbehind: true,
				greedy: true,
				inside: {
					'string': {
						pattern: /((?:^|[^\\])(?:\\\\)*)(?:"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/,
						lookbehind: true,
						inside: {
							'escape': {
								pattern: /\\["'\\nrt]/,
								alias: 'entity'
							},
							'variable': variable
						}
					},
					'comment': {
						pattern: /(\s)#.*/,
						lookbehind: true,
						greedy: true
					},
					'keyword': {
						pattern: /^\S+/,
						greedy: true
					},

					// other patterns

					'boolean': {
						pattern: /(\s)(?:off|on)(?!\S)/,
						lookbehind: true
					},
					'number': {
						pattern: /(\s)\d+[a-z]*(?!\S)/i,
						lookbehind: true
					},
					'variable': variable
				}
			},
			'punctuation': /[{};]/
		};

	}(Prism));

	Prism.languages.nim = {
		'comment': /#.*/,
		// Double-quoted strings can be prefixed by an identifier (Generalized raw string literals)
		// Character literals are handled specifically to prevent issues with numeric type suffixes
		'string': {
			pattern: /(?:(?:\b(?!\d)(?:\w|\\x[8-9a-fA-F][0-9a-fA-F])+)?(?:"""[\s\S]*?"""(?!")|"(?:\\[\s\S]|""|[^"\\])*")|'(?:\\(?:\d+|x[\da-fA-F]{2}|.)|[^'])')/,
			greedy: true
		},
		// The negative look ahead prevents wrong highlighting of the .. operator
		'number': /\b(?:0[xXoObB][\da-fA-F_]+|\d[\d_]*(?:(?!\.\.)\.[\d_]*)?(?:[eE][+-]?\d[\d_]*)?)(?:'?[iuf]\d*)?/,
		'keyword': /\b(?:addr|as|asm|atomic|bind|block|break|case|cast|concept|const|continue|converter|defer|discard|distinct|do|elif|else|end|enum|except|export|finally|for|from|func|generic|if|import|include|interface|iterator|let|macro|method|mixin|nil|object|out|proc|ptr|raise|ref|return|static|template|try|tuple|type|using|var|when|while|with|without|yield)\b/,
		'function': {
			pattern: /(?:(?!\d)(?:\w|\\x[8-9a-fA-F][0-9a-fA-F])+|`[^`\r\n]+`)\*?(?:\[[^\]]+\])?(?=\s*\()/,
			inside: {
				'operator': /\*$/
			}
		},
		// We don't want to highlight operators inside backticks
		'ignore': {
			pattern: /`[^`\r\n]+`/,
			inside: {
				'punctuation': /`/
			}
		},
		'operator': {
			// Look behind and look ahead prevent wrong highlighting of punctuations [. .] {. .} (. .)
			// but allow the slice operator .. to take precedence over them
			// One can define his own operators in Nim so all combination of operators might be an operator.
			pattern: /(^|[({\[](?=\.\.)|(?![({\[]\.).)(?:(?:[=+\-*\/<>@$~&%|!?^:\\]|\.\.|\.(?![)}\]]))+|\b(?:and|div|of|or|in|is|isnot|mod|not|notin|shl|shr|xor)\b)/m,
			lookbehind: true
		},
		'punctuation': /[({\[]\.|\.[)}\]]|[`(){}\[\],:]/
	};
	Prism.languages.objectivec = Prism.languages.extend('c', {
		'string': /("|')(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1|@"(?:\\(?:\r\n|[\s\S])|[^"\\\r\n])*"/,
		'keyword': /\b(?:asm|typeof|inline|auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|int|long|register|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while|in|self|super)\b|(?:@interface|@end|@implementation|@protocol|@class|@public|@protected|@private|@property|@try|@catch|@finally|@throw|@synthesize|@dynamic|@selector)\b/,
		'operator': /-[->]?|\+\+?|!=?|<<?=?|>>?=?|==?|&&?|\|\|?|[~^%?*\/@]/
	});

	delete Prism.languages.objectivec['class-name'];

	Prism.languages.objc = Prism.languages.objectivec;

	Prism.languages.ocaml = {
		'comment': /\(\*[\s\S]*?\*\)/,
		'string': [
			{
				pattern: /"(?:\\.|[^\\\r\n"])*"/,
				greedy: true
			},
			{
				pattern: /(['`])(?:\\(?:\d+|x[\da-f]+|.)|(?!\1)[^\\\r\n])\1/i,
				greedy: true
			}
		],
		'number': /\b(?:0x[\da-f][\da-f_]+|(?:0[bo])?\d[\d_]*(?:\.[\d_]*)?(?:e[+-]?[\d_]+)?)/i,
		'directive': {
			pattern: /\B#\w+/,
			alias: 'important'
		},
		'label': {
			pattern: /\B~\w+/,
			alias: 'function'
		},
		'type-variable': {
			pattern: /\B'\w+/,
			alias: 'function'
		},
		'variant': {
			pattern: /`\w+/,
			alias: 'variable'
		},
		'module': {
			pattern: /\b[A-Z]\w+/,
			alias: 'variable'
		},
		// For the list of keywords and operators,
		// see: http://caml.inria.fr/pub/docs/manual-ocaml/lex.html#sec84
		'keyword': /\b(?:as|assert|begin|class|constraint|do|done|downto|else|end|exception|external|for|fun|function|functor|if|in|include|inherit|initializer|lazy|let|match|method|module|mutable|new|nonrec|object|of|open|private|rec|sig|struct|then|to|try|type|val|value|virtual|when|where|while|with)\b/,
		'boolean': /\b(?:false|true)\b/,
		// Custom operators are allowed
		'operator': /:=|[=<>@^|&+\-*\/$%!?~][!$%&*+\-.\/:<=>?@^|~]*|\b(?:and|asr|land|lor|lsl|lsr|lxor|mod|or)\b/,
		'punctuation': /[(){}\[\]|.,:;]|\b_\b/
	};

	(function (Prism) {
		/* OpenCL kernel language */
		Prism.languages.opencl = Prism.languages.extend('c', {
			// Extracted from the official specs (2.0) and http://streamcomputing.eu/downloads/?opencl.lang (opencl-keywords, opencl-types) and http://sourceforge.net/tracker/?func=detail&aid=2957794&group_id=95717&atid=612384 (Words2, partly Words3)
			'keyword': /\b(?:__attribute__|(?:__)?(?:constant|global|kernel|local|private|read_only|read_write|write_only)|auto|break|case|complex|const|continue|default|do|(?:float|double)(?:16(?:x(?:1|16|2|4|8))?|1x(?:1|16|2|4|8)|2(?:x(?:1|16|2|4|8))?|3|4(?:x(?:1|16|2|4|8))?|8(?:x(?:1|16|2|4|8))?)?|else|enum|extern|for|goto|(?:u?(?:char|short|int|long)|half|quad|bool)(?:2|3|4|8|16)?|if|imaginary|inline|packed|pipe|register|restrict|return|signed|sizeof|static|struct|switch|typedef|uniform|union|unsigned|void|volatile|while)\b/,
			// Extracted from http://streamcomputing.eu/downloads/?opencl.lang (opencl-const)
			// Math Constants: https://www.khronos.org/registry/OpenCL/sdk/2.1/docs/man/xhtml/mathConstants.html
			// Macros and Limits: https://www.khronos.org/registry/OpenCL/sdk/2.1/docs/man/xhtml/macroLimits.html
			'number': /(?:\b0x(?:[\da-f]+(?:\.[\da-f]*)?|\.[\da-f]+)(?:p[+-]?\d+)?|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?)[fuhl]{0,4}/i,
			'boolean': /\b(?:false|true)\b/,
			'constant-opencl-kernel': {
				pattern: /\b(?:CHAR_(?:BIT|MAX|MIN)|CLK_(?:ADDRESS_(?:CLAMP(?:_TO_EDGE)?|NONE|REPEAT)|FILTER_(?:LINEAR|NEAREST)|(?:LOCAL|GLOBAL)_MEM_FENCE|NORMALIZED_COORDS_(?:FALSE|TRUE))|CL_(?:BGRA|(?:HALF_)?FLOAT|INTENSITY|LUMINANCE|A?R?G?B?[Ax]?|(?:(?:UN)?SIGNED|[US]NORM)_(?:INT(?:8|16|32))|UNORM_(?:INT_101010|SHORT_(?:555|565)))|(?:DBL|FLT|HALF)_(?:DIG|EPSILON|MANT_DIG|(?:MIN|MAX)(?:(?:_10)?_EXP)?)|FLT_RADIX|HUGE_VALF?|INFINITY|(?:INT|LONG|SCHAR|SHRT)_(?:MAX|MIN)|(?:UCHAR|USHRT|UINT|ULONG)_MAX|MAXFLOAT|M_(?:[12]_PI|2_SQRTPI|E|LN(?:2|10)|LOG(?:10|2)E?|PI(?:_[24])?|SQRT(?:1_2|2))(?:_F|_H)?|NAN)\b/,
				alias: 'constant'
			}
		});

		Prism.languages.insertBefore('opencl', 'class-name', {
			// https://www.khronos.org/registry/OpenCL/sdk/2.1/docs/man/xhtml/scalarDataTypes.html
			// https://www.khronos.org/registry/OpenCL/sdk/2.1/docs/man/xhtml/otherDataTypes.html
			'builtin-type': {
				pattern: /\b(?:_cl_(?:command_queue|context|device_id|event|kernel|mem|platform_id|program|sampler)|cl_(?:image_format|mem_fence_flags)|clk_event_t|event_t|image(?:1d_(?:array_|buffer_)?t|2d_(?:array_(?:depth_|msaa_depth_|msaa_)?|depth_|msaa_depth_|msaa_)?t|3d_t)|intptr_t|ndrange_t|ptrdiff_t|queue_t|reserve_id_t|sampler_t|size_t|uintptr_t)\b/,
				alias: 'keyword'
			}
		});

		var attributes = {
			// Extracted from http://streamcomputing.eu/downloads/?opencl_host.lang (opencl-types and opencl-host)
			'type-opencl-host': {
				pattern: /\b(?:cl_(?:GLenum|GLint|GLuin|addressing_mode|bitfield|bool|buffer_create_type|build_status|channel_(?:order|type)|(?:u?(?:char|short|int|long)|float|double)(?:2|3|4|8|16)?|command_(?:queue(?:_info|_properties)?|type)|context(?:_info|_properties)?|device_(?:exec_capabilities|fp_config|id|info|local_mem_type|mem_cache_type|type)|(?:event|sampler)(?:_info)?|filter_mode|half|image_info|kernel(?:_info|_work_group_info)?|map_flags|mem(?:_flags|_info|_object_type)?|platform_(?:id|info)|profiling_info|program(?:_build_info|_info)?))\b/,
				alias: 'keyword'
			},
			'boolean-opencl-host': {
				pattern: /\bCL_(?:TRUE|FALSE)\b/,
				alias: 'boolean'
			},
			// Extracted from cl.h (2.0) and http://streamcomputing.eu/downloads/?opencl_host.lang (opencl-const)
			'constant-opencl-host': {
				pattern: /\bCL_(?:A|ABGR|ADDRESS_(?:CLAMP(?:_TO_EDGE)?|MIRRORED_REPEAT|NONE|REPEAT)|ARGB|BGRA|BLOCKING|BUFFER_CREATE_TYPE_REGION|BUILD_(?:ERROR|IN_PROGRESS|NONE|PROGRAM_FAILURE|SUCCESS)|COMMAND_(?:ACQUIRE_GL_OBJECTS|BARRIER|COPY_(?:BUFFER(?:_RECT|_TO_IMAGE)?|IMAGE(?:_TO_BUFFER)?)|FILL_(?:BUFFER|IMAGE)|MAP(?:_BUFFER|_IMAGE)|MARKER|MIGRATE(?:_SVM)?_MEM_OBJECTS|NATIVE_KERNEL|NDRANGE_KERNEL|READ_(?:BUFFER(?:_RECT)?|IMAGE)|RELEASE_GL_OBJECTS|SVM_(?:FREE|MAP|MEMCPY|MEMFILL|UNMAP)|TASK|UNMAP_MEM_OBJECT|USER|WRITE_(?:BUFFER(?:_RECT)?|IMAGE))|COMPILER_NOT_AVAILABLE|COMPILE_PROGRAM_FAILURE|COMPLETE|CONTEXT_(?:DEVICES|INTEROP_USER_SYNC|NUM_DEVICES|PLATFORM|PROPERTIES|REFERENCE_COUNT)|DEPTH(?:_STENCIL)?|DEVICE_(?:ADDRESS_BITS|AFFINITY_DOMAIN_(?:L[1-4]_CACHE|NEXT_PARTITIONABLE|NUMA)|AVAILABLE|BUILT_IN_KERNELS|COMPILER_AVAILABLE|DOUBLE_FP_CONFIG|ENDIAN_LITTLE|ERROR_CORRECTION_SUPPORT|EXECUTION_CAPABILITIES|EXTENSIONS|GLOBAL_(?:MEM_(?:CACHELINE_SIZE|CACHE_SIZE|CACHE_TYPE|SIZE)|VARIABLE_PREFERRED_TOTAL_SIZE)|HOST_UNIFIED_MEMORY|IL_VERSION|IMAGE(?:2D_MAX_(?:HEIGHT|WIDTH)|3D_MAX_(?:DEPTH|HEIGHT|WIDTH)|_BASE_ADDRESS_ALIGNMENT|_MAX_ARRAY_SIZE|_MAX_BUFFER_SIZE|_PITCH_ALIGNMENT|_SUPPORT)|LINKER_AVAILABLE|LOCAL_MEM_SIZE|LOCAL_MEM_TYPE|MAX_(?:CLOCK_FREQUENCY|COMPUTE_UNITS|CONSTANT_ARGS|CONSTANT_BUFFER_SIZE|GLOBAL_VARIABLE_SIZE|MEM_ALLOC_SIZE|NUM_SUB_GROUPS|ON_DEVICE_(?:EVENTS|QUEUES)|PARAMETER_SIZE|PIPE_ARGS|READ_IMAGE_ARGS|READ_WRITE_IMAGE_ARGS|SAMPLERS|WORK_GROUP_SIZE|WORK_ITEM_DIMENSIONS|WORK_ITEM_SIZES|WRITE_IMAGE_ARGS)|MEM_BASE_ADDR_ALIGN|MIN_DATA_TYPE_ALIGN_SIZE|NAME|NATIVE_VECTOR_WIDTH_(?:CHAR|DOUBLE|FLOAT|HALF|INT|LONG|SHORT)|NOT_(?:AVAILABLE|FOUND)|OPENCL_C_VERSION|PARENT_DEVICE|PARTITION_(?:AFFINITY_DOMAIN|BY_AFFINITY_DOMAIN|BY_COUNTS|BY_COUNTS_LIST_END|EQUALLY|FAILED|MAX_SUB_DEVICES|PROPERTIES|TYPE)|PIPE_MAX_(?:ACTIVE_RESERVATIONS|PACKET_SIZE)|PLATFORM|PREFERRED_(?:GLOBAL_ATOMIC_ALIGNMENT|INTEROP_USER_SYNC|LOCAL_ATOMIC_ALIGNMENT|PLATFORM_ATOMIC_ALIGNMENT|VECTOR_WIDTH_(?:CHAR|DOUBLE|FLOAT|HALF|INT|LONG|SHORT))|PRINTF_BUFFER_SIZE|PROFILE|PROFILING_TIMER_RESOLUTION|QUEUE_(?:ON_(?:DEVICE_(?:MAX_SIZE|PREFERRED_SIZE|PROPERTIES)|HOST_PROPERTIES)|PROPERTIES)|REFERENCE_COUNT|SINGLE_FP_CONFIG|SUB_GROUP_INDEPENDENT_FORWARD_PROGRESS|SVM_(?:ATOMICS|CAPABILITIES|COARSE_GRAIN_BUFFER|FINE_GRAIN_BUFFER|FINE_GRAIN_SYSTEM)|TYPE(?:_ACCELERATOR|_ALL|_CPU|_CUSTOM|_DEFAULT|_GPU)?|VENDOR(?:_ID)?|VERSION)|DRIVER_VERSION|EVENT_(?:COMMAND_(?:EXECUTION_STATUS|QUEUE|TYPE)|CONTEXT|REFERENCE_COUNT)|EXEC_(?:KERNEL|NATIVE_KERNEL|STATUS_ERROR_FOR_EVENTS_IN_WAIT_LIST)|FILTER_(?:LINEAR|NEAREST)|FLOAT|FP_(?:CORRECTLY_ROUNDED_DIVIDE_SQRT|DENORM|FMA|INF_NAN|ROUND_TO_INF|ROUND_TO_NEAREST|ROUND_TO_ZERO|SOFT_FLOAT)|GLOBAL|HALF_FLOAT|IMAGE_(?:ARRAY_SIZE|BUFFER|DEPTH|ELEMENT_SIZE|FORMAT|FORMAT_MISMATCH|FORMAT_NOT_SUPPORTED|HEIGHT|NUM_MIP_LEVELS|NUM_SAMPLES|ROW_PITCH|SLICE_PITCH|WIDTH)|INTENSITY|INVALID_(?:ARG_INDEX|ARG_SIZE|ARG_VALUE|BINARY|BUFFER_SIZE|BUILD_OPTIONS|COMMAND_QUEUE|COMPILER_OPTIONS|CONTEXT|DEVICE|DEVICE_PARTITION_COUNT|DEVICE_QUEUE|DEVICE_TYPE|EVENT|EVENT_WAIT_LIST|GLOBAL_OFFSET|GLOBAL_WORK_SIZE|GL_OBJECT|HOST_PTR|IMAGE_DESCRIPTOR|IMAGE_FORMAT_DESCRIPTOR|IMAGE_SIZE|KERNEL|KERNEL_ARGS|KERNEL_DEFINITION|KERNEL_NAME|LINKER_OPTIONS|MEM_OBJECT|MIP_LEVEL|OPERATION|PIPE_SIZE|PLATFORM|PROGRAM|PROGRAM_EXECUTABLE|PROPERTY|QUEUE_PROPERTIES|SAMPLER|VALUE|WORK_DIMENSION|WORK_GROUP_SIZE|WORK_ITEM_SIZE)|KERNEL_(?:ARG_(?:ACCESS_(?:NONE|QUALIFIER|READ_ONLY|READ_WRITE|WRITE_ONLY)|ADDRESS_(?:CONSTANT|GLOBAL|LOCAL|PRIVATE|QUALIFIER)|INFO_NOT_AVAILABLE|NAME|TYPE_(?:CONST|NAME|NONE|PIPE|QUALIFIER|RESTRICT|VOLATILE))|ATTRIBUTES|COMPILE_NUM_SUB_GROUPS|COMPILE_WORK_GROUP_SIZE|CONTEXT|EXEC_INFO_SVM_FINE_GRAIN_SYSTEM|EXEC_INFO_SVM_PTRS|FUNCTION_NAME|GLOBAL_WORK_SIZE|LOCAL_MEM_SIZE|LOCAL_SIZE_FOR_SUB_GROUP_COUNT|MAX_NUM_SUB_GROUPS|MAX_SUB_GROUP_SIZE_FOR_NDRANGE|NUM_ARGS|PREFERRED_WORK_GROUP_SIZE_MULTIPLE|PRIVATE_MEM_SIZE|PROGRAM|REFERENCE_COUNT|SUB_GROUP_COUNT_FOR_NDRANGE|WORK_GROUP_SIZE)|LINKER_NOT_AVAILABLE|LINK_PROGRAM_FAILURE|LOCAL|LUMINANCE|MAP_(?:FAILURE|READ|WRITE|WRITE_INVALIDATE_REGION)|MEM_(?:ALLOC_HOST_PTR|ASSOCIATED_MEMOBJECT|CONTEXT|COPY_HOST_PTR|COPY_OVERLAP|FLAGS|HOST_NO_ACCESS|HOST_PTR|HOST_READ_ONLY|HOST_WRITE_ONLY|KERNEL_READ_AND_WRITE|MAP_COUNT|OBJECT_(?:ALLOCATION_FAILURE|BUFFER|IMAGE1D|IMAGE1D_ARRAY|IMAGE1D_BUFFER|IMAGE2D|IMAGE2D_ARRAY|IMAGE3D|PIPE)|OFFSET|READ_ONLY|READ_WRITE|REFERENCE_COUNT|SIZE|SVM_ATOMICS|SVM_FINE_GRAIN_BUFFER|TYPE|USES_SVM_POINTER|USE_HOST_PTR|WRITE_ONLY)|MIGRATE_MEM_OBJECT_(?:CONTENT_UNDEFINED|HOST)|MISALIGNED_SUB_BUFFER_OFFSET|NONE|NON_BLOCKING|OUT_OF_(?:HOST_MEMORY|RESOURCES)|PIPE_(?:MAX_PACKETS|PACKET_SIZE)|PLATFORM_(?:EXTENSIONS|HOST_TIMER_RESOLUTION|NAME|PROFILE|VENDOR|VERSION)|PROFILING_(?:COMMAND_(?:COMPLETE|END|QUEUED|START|SUBMIT)|INFO_NOT_AVAILABLE)|PROGRAM_(?:BINARIES|BINARY_SIZES|BINARY_TYPE(?:_COMPILED_OBJECT|_EXECUTABLE|_LIBRARY|_NONE)?|BUILD_(?:GLOBAL_VARIABLE_TOTAL_SIZE|LOG|OPTIONS|STATUS)|CONTEXT|DEVICES|IL|KERNEL_NAMES|NUM_DEVICES|NUM_KERNELS|REFERENCE_COUNT|SOURCE)|QUEUED|QUEUE_(?:CONTEXT|DEVICE|DEVICE_DEFAULT|ON_DEVICE|ON_DEVICE_DEFAULT|OUT_OF_ORDER_EXEC_MODE_ENABLE|PROFILING_ENABLE|PROPERTIES|REFERENCE_COUNT|SIZE)|R|RA|READ_(?:ONLY|WRITE)_CACHE|RG|RGB|RGBA|RGBx|RGx|RUNNING|Rx|SAMPLER_(?:ADDRESSING_MODE|CONTEXT|FILTER_MODE|LOD_MAX|LOD_MIN|MIP_FILTER_MODE|NORMALIZED_COORDS|REFERENCE_COUNT)|(?:UN)?SIGNED_INT(?:8|16|32)|SNORM_INT(?:8|16)|SUBMITTED|SUCCESS|UNORM_INT(?:16|24|8|_101010|_101010_2)|UNORM_SHORT_(?:555|565)|VERSION_(?:1_0|1_1|1_2|2_0|2_1)|sBGRA|sRGB|sRGBA|sRGBx)\b/,
				alias: 'constant'
			},
			// Extracted from cl.h (2.0) and http://streamcomputing.eu/downloads/?opencl_host.lang (opencl-host)
			'function-opencl-host': {
				pattern: /\bcl(?:BuildProgram|CloneKernel|CompileProgram|Create(?:Buffer|CommandQueue(?:WithProperties)?|Context|ContextFromType|Image|Image2D|Image3D|Kernel|KernelsInProgram|Pipe|ProgramWith(?:Binary|BuiltInKernels|IL|Source)|Sampler|SamplerWithProperties|SubBuffer|SubDevices|UserEvent)|Enqueue(?:(?:Barrier|Marker)(?:WithWaitList)?|Copy(?:Buffer(?:Rect|ToImage)?|Image(?:ToBuffer)?)|(?:Fill|Map)(?:Buffer|Image)|MigrateMemObjects|NDRangeKernel|NativeKernel|(?:Read|Write)(?:Buffer(?:Rect)?|Image)|SVM(?:Free|Map|MemFill|Memcpy|MigrateMem|Unmap)|Task|UnmapMemObject|WaitForEvents)|Finish|Flush|Get(?:CommandQueueInfo|ContextInfo|Device(?:AndHostTimer|IDs|Info)|Event(?:Profiling)?Info|ExtensionFunctionAddress(?:ForPlatform)?|HostTimer|ImageInfo|Kernel(?:ArgInfo|Info|SubGroupInfo|WorkGroupInfo)|MemObjectInfo|PipeInfo|Platform(?:IDs|Info)|Program(?:Build)?Info|SamplerInfo|SupportedImageFormats)|LinkProgram|(?:Release|Retain)(?:CommandQueue|Context|Device|Event|Kernel|MemObject|Program|Sampler)|SVM(?:Alloc|Free)|Set(?:CommandQueueProperty|DefaultDeviceCommandQueue|EventCallback|Kernel(?:Arg(?:SVMPointer)?|ExecInfo)|Kernel|MemObjectDestructorCallback|UserEventStatus)|Unload(?:Platform)?Compiler|WaitForEvents)\b/,
				alias: 'function'
			}
		};

		/* OpenCL host API */
		Prism.languages.insertBefore('c', 'keyword', attributes);

		// C++ includes everything from the OpenCL C host API plus the classes defined in cl2.h
		if (Prism.languages.cpp) {
			// Extracted from doxygen class list http://github.khronos.org/OpenCL-CLHPP/annotated.html
			attributes['type-opencl-host-cpp'] = {
				pattern: /\b(?:Buffer|BufferGL|BufferRenderGL|CommandQueue|Context|Device|DeviceCommandQueue|EnqueueArgs|Event|Image|Image1D|Image1DArray|Image1DBuffer|Image2D|Image2DArray|Image2DGL|Image3D|Image3DGL|ImageFormat|ImageGL|Kernel|KernelFunctor|LocalSpaceArg|Memory|NDRange|Pipe|Platform|Program|Sampler|SVMAllocator|SVMTraitAtomic|SVMTraitCoarse|SVMTraitFine|SVMTraitReadOnly|SVMTraitReadWrite|SVMTraitWriteOnly|UserEvent)\b/,
				alias: 'keyword'
			};

			Prism.languages.insertBefore('cpp', 'keyword', attributes);
		}
	}(Prism));

	// Based on Free Pascal

	/* TODO
		Support inline asm ?
	*/

	Prism.languages.pascal = {
		'comment': [
			/\(\*[\s\S]+?\*\)/,
			/\{[\s\S]+?\}/,
			/\/\/.*/
		],
		'string': {
			pattern: /(?:'(?:''|[^'\r\n])*'(?!')|#[&$%]?[a-f\d]+)+|\^[a-z]/i,
			greedy: true
		},
		'keyword': [
			{
				// Turbo Pascal
				pattern: /(^|[^&])\b(?:absolute|array|asm|begin|case|const|constructor|destructor|do|downto|else|end|file|for|function|goto|if|implementation|inherited|inline|interface|label|nil|object|of|operator|packed|procedure|program|record|reintroduce|repeat|self|set|string|then|to|type|unit|until|uses|var|while|with)\b/i,
				lookbehind: true
			},
			{
				// Free Pascal
				pattern: /(^|[^&])\b(?:dispose|exit|false|new|true)\b/i,
				lookbehind: true
			},
			{
				// Object Pascal
				pattern: /(^|[^&])\b(?:class|dispinterface|except|exports|finalization|finally|initialization|inline|library|on|out|packed|property|raise|resourcestring|threadvar|try)\b/i,
				lookbehind: true
			},
			{
				// Modifiers
				pattern: /(^|[^&])\b(?:absolute|abstract|alias|assembler|bitpacked|break|cdecl|continue|cppdecl|cvar|default|deprecated|dynamic|enumerator|experimental|export|external|far|far16|forward|generic|helper|implements|index|interrupt|iochecks|local|message|name|near|nodefault|noreturn|nostackframe|oldfpccall|otherwise|overload|override|pascal|platform|private|protected|public|published|read|register|reintroduce|result|safecall|saveregisters|softfloat|specialize|static|stdcall|stored|strict|unaligned|unimplemented|varargs|virtual|write)\b/i,
				lookbehind: true
			}
		],
		'number': [
			// Hexadecimal, octal and binary
			/(?:[&%]\d+|\$[a-f\d]+)/i,
			// Decimal
			/\b\d+(?:\.\d+)?(?:e[+-]?\d+)?/i
		],
		'operator': [
			/\.\.|\*\*|:=|<[<=>]?|>[>=]?|[+\-*\/]=?|[@^=]/i,
			{
				pattern: /(^|[^&])\b(?:and|as|div|exclude|in|include|is|mod|not|or|shl|shr|xor)\b/,
				lookbehind: true
			}
		],
		'punctuation': /\(\.|\.\)|[()\[\]:;,.]/
	};

	Prism.languages.objectpascal = Prism.languages.pascal;

	Prism.languages.perl = {
		'comment': [
			{
				// POD
				pattern: /(^\s*)=\w[\s\S]*?=cut.*/m,
				lookbehind: true
			},
			{
				pattern: /(^|[^\\$])#.*/,
				lookbehind: true
			}
		],
		// TODO Could be nice to handle Heredoc too.
		'string': [
			// q/.../
			{
				pattern: /\b(?:q|qq|qx|qw)\s*([^a-zA-Z0-9\s{(\[<])(?:(?!\1)[^\\]|\\[\s\S])*\1/,
				greedy: true
			},

			// q a...a
			{
				pattern: /\b(?:q|qq|qx|qw)\s+([a-zA-Z0-9])(?:(?!\1)[^\\]|\\[\s\S])*\1/,
				greedy: true
			},

			// q(...)
			{
				pattern: /\b(?:q|qq|qx|qw)\s*\((?:[^()\\]|\\[\s\S])*\)/,
				greedy: true
			},

			// q{...}
			{
				pattern: /\b(?:q|qq|qx|qw)\s*\{(?:[^{}\\]|\\[\s\S])*\}/,
				greedy: true
			},

			// q[...]
			{
				pattern: /\b(?:q|qq|qx|qw)\s*\[(?:[^[\]\\]|\\[\s\S])*\]/,
				greedy: true
			},

			// q<...>
			{
				pattern: /\b(?:q|qq|qx|qw)\s*<(?:[^<>\\]|\\[\s\S])*>/,
				greedy: true
			},

			// "...", `...`
			{
				pattern: /("|`)(?:(?!\1)[^\\]|\\[\s\S])*\1/,
				greedy: true
			},

			// '...'
			// FIXME Multi-line single-quoted strings are not supported as they would break variables containing '
			{
				pattern: /'(?:[^'\\\r\n]|\\.)*'/,
				greedy: true
			}
		],
		'regex': [
			// m/.../
			{
				pattern: /\b(?:m|qr)\s*([^a-zA-Z0-9\s{(\[<])(?:(?!\1)[^\\]|\\[\s\S])*\1[msixpodualngc]*/,
				greedy: true
			},

			// m a...a
			{
				pattern: /\b(?:m|qr)\s+([a-zA-Z0-9])(?:(?!\1)[^\\]|\\[\s\S])*\1[msixpodualngc]*/,
				greedy: true
			},

			// m(...)
			{
				pattern: /\b(?:m|qr)\s*\((?:[^()\\]|\\[\s\S])*\)[msixpodualngc]*/,
				greedy: true
			},

			// m{...}
			{
				pattern: /\b(?:m|qr)\s*\{(?:[^{}\\]|\\[\s\S])*\}[msixpodualngc]*/,
				greedy: true
			},

			// m[...]
			{
				pattern: /\b(?:m|qr)\s*\[(?:[^[\]\\]|\\[\s\S])*\][msixpodualngc]*/,
				greedy: true
			},

			// m<...>
			{
				pattern: /\b(?:m|qr)\s*<(?:[^<>\\]|\\[\s\S])*>[msixpodualngc]*/,
				greedy: true
			},

			// The lookbehinds prevent -s from breaking
			// FIXME We don't handle change of separator like s(...)[...]
			// s/.../.../
			{
				pattern: /(^|[^-]\b)(?:s|tr|y)\s*([^a-zA-Z0-9\s{(\[<])(?:(?!\2)[^\\]|\\[\s\S])*\2(?:(?!\2)[^\\]|\\[\s\S])*\2[msixpodualngcer]*/,
				lookbehind: true,
				greedy: true
			},

			// s a...a...a
			{
				pattern: /(^|[^-]\b)(?:s|tr|y)\s+([a-zA-Z0-9])(?:(?!\2)[^\\]|\\[\s\S])*\2(?:(?!\2)[^\\]|\\[\s\S])*\2[msixpodualngcer]*/,
				lookbehind: true,
				greedy: true
			},

			// s(...)(...)
			{
				pattern: /(^|[^-]\b)(?:s|tr|y)\s*\((?:[^()\\]|\\[\s\S])*\)\s*\((?:[^()\\]|\\[\s\S])*\)[msixpodualngcer]*/,
				lookbehind: true,
				greedy: true
			},

			// s{...}{...}
			{
				pattern: /(^|[^-]\b)(?:s|tr|y)\s*\{(?:[^{}\\]|\\[\s\S])*\}\s*\{(?:[^{}\\]|\\[\s\S])*\}[msixpodualngcer]*/,
				lookbehind: true,
				greedy: true
			},

			// s[...][...]
			{
				pattern: /(^|[^-]\b)(?:s|tr|y)\s*\[(?:[^[\]\\]|\\[\s\S])*\]\s*\[(?:[^[\]\\]|\\[\s\S])*\][msixpodualngcer]*/,
				lookbehind: true,
				greedy: true
			},

			// s<...><...>
			{
				pattern: /(^|[^-]\b)(?:s|tr|y)\s*<(?:[^<>\\]|\\[\s\S])*>\s*<(?:[^<>\\]|\\[\s\S])*>[msixpodualngcer]*/,
				lookbehind: true,
				greedy: true
			},

			// /.../
			// The look-ahead tries to prevent two divisions on
			// the same line from being highlighted as regex.
			// This does not support multi-line regex.
			{
				pattern: /\/(?:[^\/\\\r\n]|\\.)*\/[msixpodualngc]*(?=\s*(?:$|[\r\n,.;})&|\-+*~<>!?^]|(?:lt|gt|le|ge|eq|ne|cmp|not|and|or|xor|x)\b))/,
				greedy: true
			}
		],

		// FIXME Not sure about the handling of ::, ', and #
		'variable': [
			// ${^POSTMATCH}
			/[&*$@%]\{\^[A-Z]+\}/,
			// $^V
			/[&*$@%]\^[A-Z_]/,
			// ${...}
			/[&*$@%]#?(?=\{)/,
			// $foo
			/[&*$@%]#?(?:(?:::)*'?(?!\d)[\w$]+(?![\w$]))+(?:::)*/i,
			// $1
			/[&*$@%]\d+/,
			// $_, @_, %!
			// The negative lookahead prevents from breaking the %= operator
			/(?!%=)[$@%][!"#$%&'()*+,\-.\/:;<=>?@[\\\]^_`{|}~]/
		],
		'filehandle': {
			// <>, <FOO>, _
			pattern: /<(?![<=])\S*>|\b_\b/,
			alias: 'symbol'
		},
		'vstring': {
			// v1.2, 1.2.3
			pattern: /v\d+(?:\.\d+)*|\d+(?:\.\d+){2,}/,
			alias: 'string'
		},
		'function': {
			pattern: /sub [a-z0-9_]+/i,
			inside: {
				keyword: /sub/
			}
		},
		'keyword': /\b(?:any|break|continue|default|delete|die|do|else|elsif|eval|for|foreach|given|goto|if|last|local|my|next|our|package|print|redo|require|return|say|state|sub|switch|undef|unless|until|use|when|while)\b/,
		'number': /\b(?:0x[\dA-Fa-f](?:_?[\dA-Fa-f])*|0b[01](?:_?[01])*|(?:(?:\d(?:_?\d)*)?\.)?\d(?:_?\d)*(?:[Ee][+-]?\d+)?)\b/,
		'operator': /-[rwxoRWXOezsfdlpSbctugkTBMAC]\b|\+[+=]?|-[-=>]?|\*\*?=?|\/\/?=?|=[=~>]?|~[~=]?|\|\|?=?|&&?=?|<(?:=>?|<=?)?|>>?=?|![~=]?|[%^]=?|\.(?:=|\.\.?)?|[\\?]|\bx(?:=|\b)|\b(?:lt|gt|le|ge|eq|ne|cmp|not|and|or|xor)\b/,
		'punctuation': /[{}[\];(),:]/
	};

	(function (Prism) {

		var typeExpression = /(?:\b[a-zA-Z]\w*|[|\\[\]])+/.source;

		Prism.languages.phpdoc = Prism.languages.extend('javadoclike', {
			'parameter': {
				pattern: RegExp('(@(?:global|param|property(?:-read|-write)?|var)\\s+(?:' + typeExpression + '\\s+)?)\\$\\w+'),
				lookbehind: true
			}
		});

		Prism.languages.insertBefore('phpdoc', 'keyword', {
			'class-name': [
				{
					pattern: RegExp('(@(?:global|package|param|property(?:-read|-write)?|return|subpackage|throws|var)\\s+)' + typeExpression),
					lookbehind: true,
					inside: {
						'keyword': /\b(?:callback|resource|boolean|integer|double|object|string|array|false|float|mixed|bool|null|self|true|void|int)\b/,
						'punctuation': /[|\\[\]()]/
					}
				}
			],
		});

		Prism.languages.javadoclike.addSupport('php', Prism.languages.phpdoc);

	}(Prism));

	Prism.languages.insertBefore('php', 'variable', {
		'this': /\$this\b/,
		'global': /\$(?:_(?:SERVER|GET|POST|FILES|REQUEST|SESSION|ENV|COOKIE)|GLOBALS|HTTP_RAW_POST_DATA|argc|argv|php_errormsg|http_response_header)\b/,
		'scope': {
			pattern: /\b[\w\\]+::/,
			inside: {
				keyword: /static|self|parent/,
				punctuation: /::|\\/
			}
		}
	});
	Prism.languages.sql = {
		'comment': {
			pattern: /(^|[^\\])(?:\/\*[\s\S]*?\*\/|(?:--|\/\/|#).*)/,
			lookbehind: true
		},
		'variable': [
			{
				pattern: /@(["'`])(?:\\[\s\S]|(?!\1)[^\\])+\1/,
				greedy: true
			},
			/@[\w.$]+/
		],
		'string': {
			pattern: /(^|[^@\\])("|')(?:\\[\s\S]|(?!\2)[^\\]|\2\2)*\2/,
			greedy: true,
			lookbehind: true
		},
		'function': /\b(?:AVG|COUNT|FIRST|FORMAT|LAST|LCASE|LEN|MAX|MID|MIN|MOD|NOW|ROUND|SUM|UCASE)(?=\s*\()/i, // Should we highlight user defined functions too?
		'keyword': /\b(?:ACTION|ADD|AFTER|ALGORITHM|ALL|ALTER|ANALYZE|ANY|APPLY|AS|ASC|AUTHORIZATION|AUTO_INCREMENT|BACKUP|BDB|BEGIN|BERKELEYDB|BIGINT|BINARY|BIT|BLOB|BOOL|BOOLEAN|BREAK|BROWSE|BTREE|BULK|BY|CALL|CASCADED?|CASE|CHAIN|CHAR(?:ACTER|SET)?|CHECK(?:POINT)?|CLOSE|CLUSTERED|COALESCE|COLLATE|COLUMNS?|COMMENT|COMMIT(?:TED)?|COMPUTE|CONNECT|CONSISTENT|CONSTRAINT|CONTAINS(?:TABLE)?|CONTINUE|CONVERT|CREATE|CROSS|CURRENT(?:_DATE|_TIME|_TIMESTAMP|_USER)?|CURSOR|CYCLE|DATA(?:BASES?)?|DATE(?:TIME)?|DAY|DBCC|DEALLOCATE|DEC|DECIMAL|DECLARE|DEFAULT|DEFINER|DELAYED|DELETE|DELIMITERS?|DENY|DESC|DESCRIBE|DETERMINISTIC|DISABLE|DISCARD|DISK|DISTINCT|DISTINCTROW|DISTRIBUTED|DO|DOUBLE|DROP|DUMMY|DUMP(?:FILE)?|DUPLICATE|ELSE(?:IF)?|ENABLE|ENCLOSED|END|ENGINE|ENUM|ERRLVL|ERRORS|ESCAPED?|EXCEPT|EXEC(?:UTE)?|EXISTS|EXIT|EXPLAIN|EXTENDED|FETCH|FIELDS|FILE|FILLFACTOR|FIRST|FIXED|FLOAT|FOLLOWING|FOR(?: EACH ROW)?|FORCE|FOREIGN|FREETEXT(?:TABLE)?|FROM|FULL|FUNCTION|GEOMETRY(?:COLLECTION)?|GLOBAL|GOTO|GRANT|GROUP|HANDLER|HASH|HAVING|HOLDLOCK|HOUR|IDENTITY(?:_INSERT|COL)?|IF|IGNORE|IMPORT|INDEX|INFILE|INNER|INNODB|INOUT|INSERT|INT|INTEGER|INTERSECT|INTERVAL|INTO|INVOKER|ISOLATION|ITERATE|JOIN|KEYS?|KILL|LANGUAGE|LAST|LEAVE|LEFT|LEVEL|LIMIT|LINENO|LINES|LINESTRING|LOAD|LOCAL|LOCK|LONG(?:BLOB|TEXT)|LOOP|MATCH(?:ED)?|MEDIUM(?:BLOB|INT|TEXT)|MERGE|MIDDLEINT|MINUTE|MODE|MODIFIES|MODIFY|MONTH|MULTI(?:LINESTRING|POINT|POLYGON)|NATIONAL|NATURAL|NCHAR|NEXT|NO|NONCLUSTERED|NULLIF|NUMERIC|OFF?|OFFSETS?|ON|OPEN(?:DATASOURCE|QUERY|ROWSET)?|OPTIMIZE|OPTION(?:ALLY)?|ORDER|OUT(?:ER|FILE)?|OVER|PARTIAL|PARTITION|PERCENT|PIVOT|PLAN|POINT|POLYGON|PRECEDING|PRECISION|PREPARE|PREV|PRIMARY|PRINT|PRIVILEGES|PROC(?:EDURE)?|PUBLIC|PURGE|QUICK|RAISERROR|READS?|REAL|RECONFIGURE|REFERENCES|RELEASE|RENAME|REPEAT(?:ABLE)?|REPLACE|REPLICATION|REQUIRE|RESIGNAL|RESTORE|RESTRICT|RETURN(?:S|ING)?|REVOKE|RIGHT|ROLLBACK|ROUTINE|ROW(?:COUNT|GUIDCOL|S)?|RTREE|RULE|SAVE(?:POINT)?|SCHEMA|SECOND|SELECT|SERIAL(?:IZABLE)?|SESSION(?:_USER)?|SET(?:USER)?|SHARE|SHOW|SHUTDOWN|SIMPLE|SMALLINT|SNAPSHOT|SOME|SONAME|SQL|START(?:ING)?|STATISTICS|STATUS|STRIPED|SYSTEM_USER|TABLES?|TABLESPACE|TEMP(?:ORARY|TABLE)?|TERMINATED|TEXT(?:SIZE)?|THEN|TIME(?:STAMP)?|TINY(?:BLOB|INT|TEXT)|TOP?|TRAN(?:SACTIONS?)?|TRIGGER|TRUNCATE|TSEQUAL|TYPES?|UNBOUNDED|UNCOMMITTED|UNDEFINED|UNION|UNIQUE|UNLOCK|UNPIVOT|UNSIGNED|UPDATE(?:TEXT)?|USAGE|USE|USER|USING|VALUES?|VAR(?:BINARY|CHAR|CHARACTER|YING)|VIEW|WAITFOR|WARNINGS|WHEN|WHERE|WHILE|WITH(?: ROLLUP|IN)?|WORK|WRITE(?:TEXT)?|YEAR)\b/i,
		'boolean': /\b(?:TRUE|FALSE|NULL)\b/i,
		'number': /\b0x[\da-f]+\b|\b\d+(?:\.\d*)?|\B\.\d+\b/i,
		'operator': /[-+*\/=%^~]|&&?|\|\|?|!=?|<(?:=>?|<|>)?|>[>=]?|\b(?:AND|BETWEEN|DIV|IN|ILIKE|IS|LIKE|NOT|OR|REGEXP|RLIKE|SOUNDS LIKE|XOR)\b/i,
		'punctuation': /[;[\]()`,.]/
	};

	(function (Prism) {

		var plsql = Prism.languages.plsql = Prism.languages.extend('sql', {
			'comment': [
				/\/\*[\s\S]*?\*\//,
				/--.*/
			]
		});

		var keyword = plsql['keyword'];
		if (!Array.isArray(keyword)) {
			keyword = plsql['keyword'] = [keyword];
		}
		keyword.unshift(
			/\b(?:ACCESS|AGENT|AGGREGATE|ARRAY|ARROW|AT|ATTRIBUTE|AUDIT|AUTHID|BFILE_BASE|BLOB_BASE|BLOCK|BODY|BOTH|BOUND|BYTE|CALLING|CHAR_BASE|CHARSET(?:FORM|ID)|CLOB_BASE|COLAUTH|COLLECT|CLUSTERS?|COMPILED|COMPRESS|CONSTANT|CONSTRUCTOR|CONTEXT|CRASH|CUSTOMDATUM|DANGLING|DATE_BASE|DEFINE|DETERMINISTIC|DURATION|ELEMENT|EMPTY|EXCEPTIONS?|EXCLUSIVE|EXTERNAL|FINAL|FORALL|FORM|FOUND|GENERAL|HEAP|HIDDEN|IDENTIFIED|IMMEDIATE|INCLUDING|INCREMENT|INDICATOR|INDEXES|INDICES|INFINITE|INITIAL|ISOPEN|INSTANTIABLE|INTERFACE|INVALIDATE|JAVA|LARGE|LEADING|LENGTH|LIBRARY|LIKE[24C]|LIMITED|LONG|LOOP|MAP|MAXEXTENTS|MAXLEN|MEMBER|MINUS|MLSLABEL|MULTISET|NAME|NAN|NATIVE|NEW|NOAUDIT|NOCOMPRESS|NOCOPY|NOTFOUND|NOWAIT|NUMBER(?:_BASE)?|OBJECT|OCI(?:COLL|DATE|DATETIME|DURATION|INTERVAL|LOBLOCATOR|NUMBER|RAW|REF|REFCURSOR|ROWID|STRING|TYPE)|OFFLINE|ONLINE|ONLY|OPAQUE|OPERATOR|ORACLE|ORADATA|ORGANIZATION|ORL(?:ANY|VARY)|OTHERS|OVERLAPS|OVERRIDING|PACKAGE|PARALLEL_ENABLE|PARAMETERS?|PASCAL|PCTFREE|PIPE(?:LINED)?|PRAGMA|PRIOR|PRIVATE|RAISE|RANGE|RAW|RECORD|REF|REFERENCE|REM|REMAINDER|RESULT|RESOURCE|RETURNING|REVERSE|ROW(?:ID|NUM|TYPE)|SAMPLE|SB[124]|SEGMENT|SELF|SEPARATE|SEQUENCE|SHORT|SIZE(?:_T)?|SPARSE|SQL(?:CODE|DATA|NAME|STATE)|STANDARD|STATIC|STDDEV|STORED|STRING|STRUCT|STYLE|SUBMULTISET|SUBPARTITION|SUBSTITUTABLE|SUBTYPE|SUCCESSFUL|SYNONYM|SYSDATE|TABAUTH|TDO|THE|TIMEZONE_(?:ABBR|HOUR|MINUTE|REGION)|TRAILING|TRANSAC(?:TIONAL)?|TRUSTED|UB[124]|UID|UNDER|UNTRUSTED|VALIDATE|VALIST|VARCHAR2|VARIABLE|VARIANCE|VARRAY|VIEWS|VOID|WHENEVER|WRAPPED|ZONE)\b/i
		);

		var operator = plsql['operator'];
		if (!Array.isArray(operator)) {
			operator = plsql['operator'] = [operator];
		}
		operator.unshift(
			/:=/
		);

	}(Prism));

	(function (Prism) {

		var powershell = Prism.languages.powershell = {
			'comment': [
				{
					pattern: /(^|[^`])<#[\s\S]*?#>/,
					lookbehind: true
				},
				{
					pattern: /(^|[^`])#.*/,
					lookbehind: true
				}
			],
			'string': [
				{
					pattern: /"(?:`[\s\S]|[^`"])*"/,
					greedy: true,
					inside: {
						'function': {
							// Allow for one level of nesting
							pattern: /(^|[^`])\$\((?:\$\([^\r\n()]*\)|(?!\$\()[^\r\n)])*\)/,
							lookbehind: true,
							// Populated at end of file
							inside: {}
						}
					}
				},
				{
					pattern: /'(?:[^']|'')*'/,
					greedy: true
				}
			],
			// Matches name spaces as well as casts, attribute decorators. Force starting with letter to avoid matching array indices
			// Supports two levels of nested brackets (e.g. `[OutputType([System.Collections.Generic.List[int]])]`)
			'namespace': /\[[a-z](?:\[(?:\[[^\]]*]|[^\[\]])*]|[^\[\]])*]/i,
			'boolean': /\$(?:true|false)\b/i,
			'variable': /\$\w+\b/,
			// Cmdlets and aliases. Aliases should come last, otherwise "write" gets preferred over "write-host" for example
			// Get-Command | ?{ $_.ModuleName -match "Microsoft.PowerShell.(Util|Core|Management)" }
			// Get-Alias | ?{ $_.ReferencedCommand.Module.Name -match "Microsoft.PowerShell.(Util|Core|Management)" }
			'function': [
				/\b(?:Add|Approve|Assert|Backup|Block|Checkpoint|Clear|Close|Compare|Complete|Compress|Confirm|Connect|Convert|ConvertFrom|ConvertTo|Copy|Debug|Deny|Disable|Disconnect|Dismount|Edit|Enable|Enter|Exit|Expand|Export|Find|ForEach|Format|Get|Grant|Group|Hide|Import|Initialize|Install|Invoke|Join|Limit|Lock|Measure|Merge|Move|New|Open|Optimize|Out|Ping|Pop|Protect|Publish|Push|Read|Receive|Redo|Register|Remove|Rename|Repair|Request|Reset|Resize|Resolve|Restart|Restore|Resume|Revoke|Save|Search|Select|Send|Set|Show|Skip|Sort|Split|Start|Step|Stop|Submit|Suspend|Switch|Sync|Tee|Test|Trace|Unblock|Undo|Uninstall|Unlock|Unprotect|Unpublish|Unregister|Update|Use|Wait|Watch|Where|Write)-[a-z]+\b/i,
				/\b(?:ac|cat|chdir|clc|cli|clp|clv|compare|copy|cp|cpi|cpp|cvpa|dbp|del|diff|dir|ebp|echo|epal|epcsv|epsn|erase|fc|fl|ft|fw|gal|gbp|gc|gci|gcs|gdr|gi|gl|gm|gp|gps|group|gsv|gu|gv|gwmi|iex|ii|ipal|ipcsv|ipsn|irm|iwmi|iwr|kill|lp|ls|measure|mi|mount|move|mp|mv|nal|ndr|ni|nv|ogv|popd|ps|pushd|pwd|rbp|rd|rdr|ren|ri|rm|rmdir|rni|rnp|rp|rv|rvpa|rwmi|sal|saps|sasv|sbp|sc|select|set|shcm|si|sl|sleep|sls|sort|sp|spps|spsv|start|sv|swmi|tee|trcm|type|write)\b/i
			],
			// per http://technet.microsoft.com/en-us/library/hh847744.aspx
			'keyword': /\b(?:Begin|Break|Catch|Class|Continue|Data|Define|Do|DynamicParam|Else|ElseIf|End|Exit|Filter|Finally|For|ForEach|From|Function|If|InlineScript|Parallel|Param|Process|Return|Sequence|Switch|Throw|Trap|Try|Until|Using|Var|While|Workflow)\b/i,
			'operator': {
				pattern: /(\W?)(?:!|-(?:eq|ne|gt|ge|lt|le|sh[lr]|not|b?(?:and|x?or)|(?:Not)?(?:Like|Match|Contains|In)|Replace|Join|is(?:Not)?|as)\b|-[-=]?|\+[+=]?|[*\/%]=?)/i,
				lookbehind: true
			},
			'punctuation': /[|{}[\];(),.]/
		};

		// Variable interpolation inside strings, and nested expressions
		var stringInside = powershell.string[0].inside;
		stringInside.boolean = powershell.boolean;
		stringInside.variable = powershell.variable;
		stringInside.function.inside = powershell;

	}(Prism));

	Prism.languages.purescript = Prism.languages.extend('haskell', {
		'keyword': /\b(?:ado|case|class|data|derive|do|else|forall|if|in|infixl|infixr|instance|let|module|newtype|of|primitive|then|type|where)\b/,

		'import-statement': {
			// The imported or hidden names are not included in this import
			// statement. This is because we want to highlight those exactly like
			// we do for the names in the program.
			pattern: /(^\s*)import\s+[A-Z][\w']*(?:\.[A-Z][\w']*)*(?:\s+as\s+[A-Z][\w']*(?:\.[A-Z][\w']*)*)?(?:\s+hiding\b)?/m,
			lookbehind: true,
			inside: {
				'keyword': /\b(?:import|as|hiding)\b/
			}
		},

		// These are builtin functions only. Constructors are highlighted later as a constant.
		'builtin': /\b(?:absurd|add|ap|append|apply|between|bind|bottom|clamp|compare|comparing|compose|conj|const|degree|discard|disj|div|eq|flap|flip|gcd|identity|ifM|join|lcm|liftA1|liftM1|map|max|mempty|min|mod|mul|negate|not|notEq|one|otherwise|recip|show|sub|top|unit|unless|unlessM|void|when|whenM|zero)\b/,
	});

	Prism.languages.purs = Prism.languages.purescript;

	Prism.languages.python = {
		'comment': {
			pattern: /(^|[^\\])#.*/,
			lookbehind: true
		},
		'string-interpolation': {
			pattern: /(?:f|rf|fr)(?:("""|''')[\s\S]*?\1|("|')(?:\\.|(?!\2)[^\\\r\n])*\2)/i,
			greedy: true,
			inside: {
				'interpolation': {
					// "{" <expression> <optional "!s", "!r", or "!a"> <optional ":" format specifier> "}"
					pattern: /((?:^|[^{])(?:{{)*){(?!{)(?:[^{}]|{(?!{)(?:[^{}]|{(?!{)(?:[^{}])+})+})+}/,
					lookbehind: true,
					inside: {
						'format-spec': {
							pattern: /(:)[^:(){}]+(?=}$)/,
							lookbehind: true
						},
						'conversion-option': {
							pattern: /![sra](?=[:}]$)/,
							alias: 'punctuation'
						},
						rest: null
					}
				},
				'string': /[\s\S]+/
			}
		},
		'triple-quoted-string': {
			pattern: /(?:[rub]|rb|br)?("""|''')[\s\S]*?\1/i,
			greedy: true,
			alias: 'string'
		},
		'string': {
			pattern: /(?:[rub]|rb|br)?("|')(?:\\.|(?!\1)[^\\\r\n])*\1/i,
			greedy: true
		},
		'function': {
			pattern: /((?:^|\s)def[ \t]+)[a-zA-Z_]\w*(?=\s*\()/g,
			lookbehind: true
		},
		'class-name': {
			pattern: /(\bclass\s+)\w+/i,
			lookbehind: true
		},
		'decorator': {
			pattern: /(^\s*)@\w+(?:\.\w+)*/im,
			lookbehind: true,
			alias: ['annotation', 'punctuation'],
			inside: {
				'punctuation': /\./
			}
		},
		'keyword': /\b(?:and|as|assert|async|await|break|class|continue|def|del|elif|else|except|exec|finally|for|from|global|if|import|in|is|lambda|nonlocal|not|or|pass|print|raise|return|try|while|with|yield)\b/,
		'builtin': /\b(?:__import__|abs|all|any|apply|ascii|basestring|bin|bool|buffer|bytearray|bytes|callable|chr|classmethod|cmp|coerce|compile|complex|delattr|dict|dir|divmod|enumerate|eval|execfile|file|filter|float|format|frozenset|getattr|globals|hasattr|hash|help|hex|id|input|int|intern|isinstance|issubclass|iter|len|list|locals|long|map|max|memoryview|min|next|object|oct|open|ord|pow|property|range|raw_input|reduce|reload|repr|reversed|round|set|setattr|slice|sorted|staticmethod|str|sum|super|tuple|type|unichr|unicode|vars|xrange|zip)\b/,
		'boolean': /\b(?:True|False|None)\b/,
		'number': /(?:\b(?=\d)|\B(?=\.))(?:0[bo])?(?:(?:\d|0x[\da-f])[\da-f]*(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?j?\b/i,
		'operator': /[-+%=]=?|!=|\*\*?=?|\/\/?=?|<[<=>]?|>[=>]?|[&|^~]/,
		'punctuation': /[{}[\];(),.:]/
	};

	Prism.languages.python['string-interpolation'].inside['interpolation'].inside.rest = Prism.languages.python;

	Prism.languages.py = Prism.languages.python;

	(function (Prism) {

		var jsString = /"(?:\\.|[^\\"\r\n])*"|'(?:\\.|[^\\'\r\n])*'/.source;
		var jsComment = /\/\/.*(?!.)|\/\*(?:[^*]|\*(?!\/))*\*\//.source;

		var jsExpr = /(?:[^\\()[\]{}"'/]|<string>|\/(?![*/])|<comment>|\(<expr>*\)|\[<expr>*\]|\{<expr>*\}|\\[\s\S])/
			.source.replace(/<string>/g, function () { return jsString; }).replace(/<comment>/g, function () { return jsComment; });

		// the pattern will blow up, so only a few iterations
		for (var i = 0; i < 2; i++) {
			jsExpr = jsExpr.replace(/<expr>/g, function () { return jsExpr; });
		}
		jsExpr = jsExpr.replace(/<expr>/g, '[^\\s\\S]');


		Prism.languages.qml = {
			'comment': {
				pattern: /\/\/.*|\/\*[\s\S]*?\*\//,
				greedy: true
			},
			'javascript-function': {
				pattern: RegExp(/((?:^|;)[ \t]*)function\s+(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*\s*\(<js>*\)\s*\{<js>*\}/.source.replace(/<js>/g, function () { return jsExpr; }), 'm'),
				lookbehind: true,
				greedy: true,
				alias: 'language-javascript',
				inside: Prism.languages.javascript
			},
			'class-name': {
				pattern: /((?:^|[:;])[ \t]*)(?!\d)\w+(?=[ \t]*\{|[ \t]+on\b)/m,
				lookbehind: true
			},
			'property': [
				{
					pattern: /((?:^|[;{])[ \t]*)(?!\d)\w+(?:\.\w+)*(?=[ \t]*:)/m,
					lookbehind: true
				},
				{
					pattern: /((?:^|[;{])[ \t]*)property[ \t]+(?!\d)\w+(?:\.\w+)*[ \t]+(?!\d)\w+(?:\.\w+)*(?=[ \t]*:)/m,
					lookbehind: true,
					inside: {
						'keyword': /^property/,
						'property': /\w+(?:\.\w+)*/
					}
				}
			],
			'javascript-expression': {
				pattern: RegExp(/(:[ \t]*)(?![\s;}[])(?:(?!$|[;}])<js>)+/.source.replace(/<js>/g, function () { return jsExpr; }), 'm'),
				lookbehind: true,
				greedy: true,
				alias: 'language-javascript',
				inside: Prism.languages.javascript
			},
			'string': /"(?:\\.|[^\\"\r\n])*"/,
			'keyword': /\b(?:as|import|on)\b/,
			'punctuation': /[{}[\]:;,]/
		};

	}(Prism));

	Prism.languages.r = {
		'comment': /#.*/,
		'string': {
			pattern: /(['"])(?:\\.|(?!\1)[^\\\r\n])*\1/,
			greedy: true
		},
		'percent-operator': {
			// Includes user-defined operators
			// and %%, %*%, %/%, %in%, %o%, %x%
			pattern: /%[^%\s]*%/,
			alias: 'operator'
		},
		'boolean': /\b(?:TRUE|FALSE)\b/,
		'ellipsis': /\.\.(?:\.|\d+)/,
		'number': [
			/\b(?:NaN|Inf)\b/,
			/(?:\b0x[\dA-Fa-f]+(?:\.\d*)?|\b\d+(?:\.\d*)?|\B\.\d+)(?:[EePp][+-]?\d+)?[iL]?/
		],
		'keyword': /\b(?:if|else|repeat|while|function|for|in|next|break|NULL|NA|NA_integer_|NA_real_|NA_complex_|NA_character_)\b/,
		'operator': /->?>?|<(?:=|<?-)?|[>=!]=?|::?|&&?|\|\|?|[+*\/^$@~]/,
		'punctuation': /[(){}\[\],;]/
	};

	(function (Prism) {

	var javascript = Prism.util.clone(Prism.languages.javascript);

	var space = /(?:\s|\/\/.*(?!.)|\/\*(?:[^*]|\*(?!\/))\*\/)/.source;
	var braces = /(?:\{(?:\{(?:\{[^{}]*\}|[^{}])*\}|[^{}])*\})/.source;
	var spread = /(?:\{<S>*\.{3}(?:[^{}]|<BRACES>)*\})/.source;

	/**
	 * @param {string} source
	 * @param {string} [flags]
	 */
	function re(source, flags) {
		source = source
			.replace(/<S>/g, function () { return space; })
			.replace(/<BRACES>/g, function () { return braces; })
			.replace(/<SPREAD>/g, function () { return spread; });
		return RegExp(source, flags);
	}

	spread = re(spread).source;


	Prism.languages.jsx = Prism.languages.extend('markup', javascript);
	Prism.languages.jsx.tag.pattern = re(
		/<\/?(?:[\w.:-]+(?:<S>+(?:[\w.:$-]+(?:=(?:"(?:\\[^]|[^\\"])*"|'(?:\\[^]|[^\\'])*'|[^\s{'"/>=]+|<BRACES>))?|<SPREAD>))*<S>*\/?)?>/.source
	);

	Prism.languages.jsx.tag.inside['tag'].pattern = /^<\/?[^\s>\/]*/i;
	Prism.languages.jsx.tag.inside['attr-value'].pattern = /=(?!\{)(?:"(?:\\[^]|[^\\"])*"|'(?:\\[^]|[^\\'])*'|[^\s'">]+)/i;
	Prism.languages.jsx.tag.inside['tag'].inside['class-name'] = /^[A-Z]\w*(?:\.[A-Z]\w*)*$/;
	Prism.languages.jsx.tag.inside['comment'] = javascript['comment'];

	Prism.languages.insertBefore('inside', 'attr-name', {
		'spread': {
			pattern: re(/<SPREAD>/.source),
			inside: Prism.languages.jsx
		}
	}, Prism.languages.jsx.tag);

	Prism.languages.insertBefore('inside', 'special-attr', {
		'script': {
			// Allow for two levels of nesting
			pattern: re(/=<BRACES>/.source),
			inside: {
				'script-punctuation': {
					pattern: /^=(?={)/,
					alias: 'punctuation'
				},
				rest: Prism.languages.jsx
			},
			'alias': 'language-javascript'
		}
	}, Prism.languages.jsx.tag);

	// The following will handle plain text inside tags
	var stringifyToken = function (token) {
		if (!token) {
			return '';
		}
		if (typeof token === 'string') {
			return token;
		}
		if (typeof token.content === 'string') {
			return token.content;
		}
		return token.content.map(stringifyToken).join('');
	};

	var walkTokens = function (tokens) {
		var openedTags = [];
		for (var i = 0; i < tokens.length; i++) {
			var token = tokens[i];
			var notTagNorBrace = false;

			if (typeof token !== 'string') {
				if (token.type === 'tag' && token.content[0] && token.content[0].type === 'tag') {
					// We found a tag, now find its kind

					if (token.content[0].content[0].content === '</') {
						// Closing tag
						if (openedTags.length > 0 && openedTags[openedTags.length - 1].tagName === stringifyToken(token.content[0].content[1])) {
							// Pop matching opening tag
							openedTags.pop();
						}
					} else {
						if (token.content[token.content.length - 1].content === '/>') {
							// Autoclosed tag, ignore
						} else {
							// Opening tag
							openedTags.push({
								tagName: stringifyToken(token.content[0].content[1]),
								openedBraces: 0
							});
						}
					}
				} else if (openedTags.length > 0 && token.type === 'punctuation' && token.content === '{') {

					// Here we might have entered a JSX context inside a tag
					openedTags[openedTags.length - 1].openedBraces++;

				} else if (openedTags.length > 0 && openedTags[openedTags.length - 1].openedBraces > 0 && token.type === 'punctuation' && token.content === '}') {

					// Here we might have left a JSX context inside a tag
					openedTags[openedTags.length - 1].openedBraces--;

				} else {
					notTagNorBrace = true;
				}
			}
			if (notTagNorBrace || typeof token === 'string') {
				if (openedTags.length > 0 && openedTags[openedTags.length - 1].openedBraces === 0) {
					// Here we are inside a tag, and not inside a JSX context.
					// That's plain text: drop any tokens matched.
					var plainText = stringifyToken(token);

					// And merge text with adjacent text
					if (i < tokens.length - 1 && (typeof tokens[i + 1] === 'string' || tokens[i + 1].type === 'plain-text')) {
						plainText += stringifyToken(tokens[i + 1]);
						tokens.splice(i + 1, 1);
					}
					if (i > 0 && (typeof tokens[i - 1] === 'string' || tokens[i - 1].type === 'plain-text')) {
						plainText = stringifyToken(tokens[i - 1]) + plainText;
						tokens.splice(i - 1, 1);
						i--;
					}

					tokens[i] = new Prism.Token('plain-text', plainText, null, plainText);
				}
			}

			if (token.content && typeof token.content !== 'string') {
				walkTokens(token.content);
			}
		}
	};

	Prism.hooks.add('after-tokenize', function (env) {
		if (env.language !== 'jsx' && env.language !== 'tsx') {
			return;
		}
		walkTokens(env.tokens);
	});

	}(Prism));

	(function (Prism) {
		var typescript = Prism.util.clone(Prism.languages.typescript);
		Prism.languages.tsx = Prism.languages.extend('jsx', typescript);

		// This will prevent collisions between TSX tags and TS generic types.
		// Idea by https://github.com/karlhorky
		// Discussion: https://github.com/PrismJS/prism/issues/2594#issuecomment-710666928
		var tag = Prism.languages.tsx.tag;
		tag.pattern = RegExp(/(^|[^\w$]|(?=<\/))/.source + '(?:' + tag.pattern.source + ')', tag.pattern.flags);
		tag.lookbehind = true;
	}(Prism));

	(function (Prism) {

		var specialEscape = {
			pattern: /\\[\\(){}[\]^$+*?|.]/,
			alias: 'escape'
		};
		var escape = /\\(?:x[\da-fA-F]{2}|u[\da-fA-F]{4}|u\{[\da-fA-F]+\}|c[a-zA-Z]|0[0-7]{0,2}|[123][0-7]{2}|.)/;
		var charClass = {
			pattern: /\.|\\[wsd]|\\p{[^{}]+}/i,
			alias: 'class-name'
		};
		var charClassWithoutDot = {
			pattern: /\\[wsd]|\\p{[^{}]+}/i,
			alias: 'class-name'
		};

		var rangeChar = '(?:[^\\\\-]|' + escape.source + ')';
		var range = RegExp(rangeChar + '-' + rangeChar);

		// the name of a capturing group
		var groupName = {
			pattern: /(<|')[^<>']+(?=[>']$)/,
			lookbehind: true,
			alias: 'variable'
		};

		Prism.languages.regex = {
			'charset': {
				pattern: /((?:^|[^\\])(?:\\\\)*)\[(?:[^\\\]]|\\[\s\S])*\]/,
				lookbehind: true,
				inside: {
					'charset-negation': {
						pattern: /(^\[)\^/,
						lookbehind: true,
						alias: 'operator'
					},
					'charset-punctuation': {
						pattern: /^\[|\]$/,
						alias: 'punctuation'
					},
					'range': {
						pattern: range,
						inside: {
							'escape': escape,
							'range-punctuation': {
								pattern: /-/,
								alias: 'operator'
							}
						}
					},
					'special-escape': specialEscape,
					'charclass': charClassWithoutDot,
					'escape': escape
				}
			},
			'special-escape': specialEscape,
			'charclass': charClass,
			'backreference': [
				{
					// a backreference which is not an octal escape
					pattern: /\\(?![123][0-7]{2})[1-9]/,
					alias: 'keyword'
				},
				{
					pattern: /\\k<[^<>']+>/,
					alias: 'keyword',
					inside: {
						'group-name': groupName
					}
				}
			],
			'anchor': {
				pattern: /[$^]|\\[ABbGZz]/,
				alias: 'function'
			},
			'escape': escape,
			'group': [
				{
					// https://docs.oracle.com/javase/10/docs/api/java/util/regex/Pattern.html
					// https://docs.microsoft.com/en-us/dotnet/standard/base-types/regular-expression-language-quick-reference?view=netframework-4.7.2#grouping-constructs

					// (), (?<name>), (?'name'), (?>), (?:), (?=), (?!), (?<=), (?<!), (?is-m), (?i-m:)
					pattern: /\((?:\?(?:<[^<>']+>|'[^<>']+'|[>:]|<?[=!]|[idmnsuxU]+(?:-[idmnsuxU]+)?:?))?/,
					alias: 'punctuation',
					inside: {
						'group-name': groupName
					}
				},
				{
					pattern: /\)/,
					alias: 'punctuation'
				}
			],
			'quantifier': {
				pattern: /(?:[+*?]|\{\d+(?:,\d*)?\})[?+]?/,
				alias: 'number'
			},
			'alternation': {
				pattern: /\|/,
				alias: 'keyword'
			}
		};

	}(Prism));

	Prism.languages.rest = {
		'table': [
			{
				pattern: /(\s*)(?:\+[=-]+)+\+(?:\r?\n|\r)(?:\1[+|].+[+|](?:\r?\n|\r))+\1(?:\+[=-]+)+\+/,
				lookbehind: true,
				inside: {
					'punctuation': /\||(?:\+[=-]+)+\+/
				}
			},
			{
				pattern: /(\s*)=+ [ =]*=(?:(?:\r?\n|\r)\1.+)+(?:\r?\n|\r)\1=+ [ =]*=(?=(?:\r?\n|\r){2}|\s*$)/,
				lookbehind: true,
				inside: {
					'punctuation': /[=-]+/
				}
			}
		],

		// Directive-like patterns

		'substitution-def': {
			pattern: /(^\s*\.\. )\|(?:[^|\s](?:[^|]*[^|\s])?)\| [^:]+::/m,
			lookbehind: true,
			inside: {
				'substitution': {
					pattern: /^\|(?:[^|\s]|[^|\s][^|]*[^|\s])\|/,
					alias: 'attr-value',
					inside: {
						'punctuation': /^\||\|$/
					}
				},
				'directive': {
					pattern: /( +)(?! )[^:]+::/,
					lookbehind: true,
					alias: 'function',
					inside: {
						'punctuation': /::$/
					}
				}
			}
		},
		'link-target': [
			{
				pattern: /(^\s*\.\. )\[[^\]]+\]/m,
				lookbehind: true,
				alias: 'string',
				inside: {
					'punctuation': /^\[|\]$/
				}
			},
			{
				pattern: /(^\s*\.\. )_(?:`[^`]+`|(?:[^:\\]|\\.)+):/m,
				lookbehind: true,
				alias: 'string',
				inside: {
					'punctuation': /^_|:$/
				}
			}
		],
		'directive': {
			pattern: /(^\s*\.\. )[^:]+::/m,
			lookbehind: true,
			alias: 'function',
			inside: {
				'punctuation': /::$/
			}
		},
		'comment': {
			// The two alternatives try to prevent highlighting of blank comments
			pattern: /(^\s*\.\.)(?:(?: .+)?(?:(?:\r?\n|\r).+)+| .+)(?=(?:\r?\n|\r){2}|$)/m,
			lookbehind: true
		},

		'title': [
			// Overlined and underlined
			{
				pattern: /^(([!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~])\2+)(?:\r?\n|\r).+(?:\r?\n|\r)\1$/m,
				inside: {
					'punctuation': /^[!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~]+|[!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~]+$/,
					'important': /.+/
				}
			},

			// Underlined only
			{
				pattern: /(^|(?:\r?\n|\r){2}).+(?:\r?\n|\r)([!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~])\2+(?=\r?\n|\r|$)/,
				lookbehind: true,
				inside: {
					'punctuation': /[!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~]+$/,
					'important': /.+/
				}
			}
		],
		'hr': {
			pattern: /((?:\r?\n|\r){2})([!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~])\2{3,}(?=(?:\r?\n|\r){2})/,
			lookbehind: true,
			alias: 'punctuation'
		},
		'field': {
			pattern: /(^\s*):[^:\r\n]+:(?= )/m,
			lookbehind: true,
			alias: 'attr-name'
		},
		'command-line-option': {
			pattern: /(^\s*)(?:[+-][a-z\d]|(?:--|\/)[a-z\d-]+)(?:[ =](?:[a-z][\w-]*|<[^<>]+>))?(?:, (?:[+-][a-z\d]|(?:--|\/)[a-z\d-]+)(?:[ =](?:[a-z][\w-]*|<[^<>]+>))?)*(?=(?:\r?\n|\r)? {2,}\S)/im,
			lookbehind: true,
			alias: 'symbol'
		},
		'literal-block': {
			pattern: /::(?:\r?\n|\r){2}([ \t]+)(?![ \t]).+(?:(?:\r?\n|\r)\1.+)*/,
			inside: {
				'literal-block-punctuation': {
					pattern: /^::/,
					alias: 'punctuation'
				}
			}
		},
		'quoted-literal-block': {
			pattern: /::(?:\r?\n|\r){2}([!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~]).*(?:(?:\r?\n|\r)\1.*)*/,
			inside: {
				'literal-block-punctuation': {
					pattern: /^(?:::|([!"#$%&'()*+,\-.\/:;<=>?@\[\\\]^_`{|}~])\1*)/m,
					alias: 'punctuation'
				}
			}
		},
		'list-bullet': {
			pattern: /(^\s*)(?:[*+\-•‣⁃]|\(?(?:\d+|[a-z]|[ivxdclm]+)\)|(?:\d+|[a-z]|[ivxdclm]+)\.)(?= )/im,
			lookbehind: true,
			alias: 'punctuation'
		},
		'doctest-block': {
			pattern: /(^\s*)>>> .+(?:(?:\r?\n|\r).+)*/m,
			lookbehind: true,
			inside: {
				'punctuation': /^>>>/
			}
		},

		'inline': [
			{
				pattern: /(^|[\s\-:\/'"<(\[{])(?::[^:]+:`.*?`|`.*?`:[^:]+:|(\*\*?|``?|\|)(?!\s).*?[^\s]\2(?=[\s\-.,:;!?\\\/'")\]}]|$))/m,
				lookbehind: true,
				inside: {
					'bold': {
						pattern: /(^\*\*).+(?=\*\*$)/,
						lookbehind: true
					},
					'italic': {
						pattern: /(^\*).+(?=\*$)/,
						lookbehind: true
					},
					'inline-literal': {
						pattern: /(^``).+(?=``$)/,
						lookbehind: true,
						alias: 'symbol'
					},
					'role': {
						pattern: /^:[^:]+:|:[^:]+:$/,
						alias: 'function',
						inside: {
							'punctuation': /^:|:$/
						}
					},
					'interpreted-text': {
						pattern: /(^`).+(?=`$)/,
						lookbehind: true,
						alias: 'attr-value'
					},
					'substitution': {
						pattern: /(^\|).+(?=\|$)/,
						lookbehind: true,
						alias: 'attr-value'
					},
					'punctuation': /\*\*?|``?|\|/
				}
			}
		],

		'link': [
			{
				pattern: /\[[^\]]+\]_(?=[\s\-.,:;!?\\\/'")\]}]|$)/,
				alias: 'string',
				inside: {
					'punctuation': /^\[|\]_$/
				}
			},
			{
				pattern: /(?:\b[a-z\d]+(?:[_.:+][a-z\d]+)*_?_|`[^`]+`_?_|_`[^`]+`)(?=[\s\-.,:;!?\\\/'")\]}]|$)/i,
				alias: 'string',
				inside: {
					'punctuation': /^_?`|`$|`?_?_$/
				}
			}
		],

		// Line block start,
		// quote attribution,
		// explicit markup start,
		// and anonymous hyperlink target shortcut (__)
		'punctuation': {
			pattern: /(^\s*)(?:\|(?= |$)|(?:---?|—|\.\.|__)(?= )|\.\.$)/m,
			lookbehind: true
		}
	};

	(function (Prism) {

		var multilineComment = /\/\*(?:[^*/]|\*(?!\/)|\/(?!\*)|<self>)*\*\//.source;
		for (var i = 0; i < 2; i++) {
			// support 4 levels of nested comments
			multilineComment = multilineComment.replace(/<self>/g, function () { return multilineComment; });
		}
		multilineComment = multilineComment.replace(/<self>/g, function () { return /[^\s\S]/.source; });


		Prism.languages.rust = {
			'comment': [
				{
					pattern: RegExp(/(^|[^\\])/.source + multilineComment),
					lookbehind: true,
					greedy: true
				},
				{
					pattern: /(^|[^\\:])\/\/.*/,
					lookbehind: true,
					greedy: true
				}
			],
			'string': {
				pattern: /b?"(?:\\[\s\S]|[^\\"])*"|b?r(#*)"(?:[^"]|"(?!\1))*"\1/,
				greedy: true
			},
			'char': {
				pattern: /b?'(?:\\(?:x[0-7][\da-fA-F]|u\{(?:[\da-fA-F]_*){1,6}\}|.)|[^\\\r\n\t'])'/,
				greedy: true,
				alias: 'string'
			},
			'attribute': {
				pattern: /#!?\[(?:[^\[\]"]|"(?:\\[\s\S]|[^\\"])*")*\]/,
				greedy: true,
				alias: 'attr-name',
				inside: {
					'string': null // see below
				}
			},

			// Closure params should not be confused with bitwise OR |
			'closure-params': {
				pattern: /([=(,:]\s*|\bmove\s*)\|[^|]*\||\|[^|]*\|(?=\s*(?:\{|->))/,
				lookbehind: true,
				greedy: true,
				inside: {
					'closure-punctuation': {
						pattern: /^\||\|$/,
						alias: 'punctuation'
					},
					rest: null // see below
				}
			},

			'lifetime-annotation': {
				pattern: /'\w+/,
				alias: 'symbol'
			},

			'fragment-specifier': {
				pattern: /(\$\w+:)[a-z]+/,
				lookbehind: true,
				alias: 'punctuation'
			},
			'variable': /\$\w+/,

			'function-definition': {
				pattern: /(\bfn\s+)\w+/,
				lookbehind: true,
				alias: 'function'
			},
			'type-definition': {
				pattern: /(\b(?:enum|struct|union)\s+)\w+/,
				lookbehind: true,
				alias: 'class-name'
			},
			'module-declaration': [
				{
					pattern: /(\b(?:crate|mod)\s+)[a-z][a-z_\d]*/,
					lookbehind: true,
					alias: 'namespace'
				},
				{
					pattern: /(\b(?:crate|self|super)\s*)::\s*[a-z][a-z_\d]*\b(?:\s*::(?:\s*[a-z][a-z_\d]*\s*::)*)?/,
					lookbehind: true,
					alias: 'namespace',
					inside: {
						'punctuation': /::/
					}
				}
			],
			'keyword': [
				// https://github.com/rust-lang/reference/blob/master/src/keywords.md
				/\b(?:abstract|as|async|await|become|box|break|const|continue|crate|do|dyn|else|enum|extern|final|fn|for|if|impl|in|let|loop|macro|match|mod|move|mut|override|priv|pub|ref|return|self|Self|static|struct|super|trait|try|type|typeof|union|unsafe|unsized|use|virtual|where|while|yield)\b/,
				// primitives and str
				// https://doc.rust-lang.org/stable/rust-by-example/primitives.html
				/\b(?:[ui](?:8|16|32|64|128|size)|f(?:32|64)|bool|char|str)\b/
			],

			// functions can technically start with an upper-case letter, but this will introduce a lot of false positives
			// and Rust's naming conventions recommend snake_case anyway.
			// https://doc.rust-lang.org/1.0.0/style/style/naming/README.html
			'function': /\b[a-z_]\w*(?=\s*(?:::\s*<|\())/,
			'macro': {
				pattern: /\w+!/,
				alias: 'property'
			},
			'constant': /\b[A-Z_][A-Z_\d]+\b/,
			'class-name': /\b[A-Z]\w*\b/,

			'namespace': {
				pattern: /(?:\b[a-z][a-z_\d]*\s*::\s*)*\b[a-z][a-z_\d]*\s*::(?!\s*<)/,
				inside: {
					'punctuation': /::/
				}
			},

			// Hex, oct, bin, dec numbers with visual separators and type suffix
			'number': /\b(?:0x[\dA-Fa-f](?:_?[\dA-Fa-f])*|0o[0-7](?:_?[0-7])*|0b[01](?:_?[01])*|(?:(?:\d(?:_?\d)*)?\.)?\d(?:_?\d)*(?:[Ee][+-]?\d+)?)(?:_?(?:[iu](?:8|16|32|64|size)?|f32|f64))?\b/,
			'boolean': /\b(?:false|true)\b/,
			'punctuation': /->|\.\.=|\.{1,3}|::|[{}[\];(),:]/,
			'operator': /[-+*\/%!^]=?|=[=>]?|&[&=]?|\|[|=]?|<<?=?|>>?=?|[@?]/
		};

		Prism.languages.rust['closure-params'].inside.rest = Prism.languages.rust;
		Prism.languages.rust['attribute'].inside['string'] = Prism.languages.rust['string'];

	}(Prism));

	(function (Prism) {
		Prism.languages.sass = Prism.languages.extend('css', {
			// Sass comments don't need to be closed, only indented
			'comment': {
				pattern: /^([ \t]*)\/[\/*].*(?:(?:\r?\n|\r)\1[ \t].+)*/m,
				lookbehind: true
			}
		});

		Prism.languages.insertBefore('sass', 'atrule', {
			// We want to consume the whole line
			'atrule-line': {
				// Includes support for = and + shortcuts
				pattern: /^(?:[ \t]*)[@+=].+/m,
				inside: {
					'atrule': /(?:@[\w-]+|[+=])/m
				}
			}
		});
		delete Prism.languages.sass.atrule;


		var variable = /\$[-\w]+|#\{\$[-\w]+\}/;
		var operator = [
			/[+*\/%]|[=!]=|<=?|>=?|\b(?:and|or|not)\b/,
			{
				pattern: /(\s+)-(?=\s)/,
				lookbehind: true
			}
		];

		Prism.languages.insertBefore('sass', 'property', {
			// We want to consume the whole line
			'variable-line': {
				pattern: /^[ \t]*\$.+/m,
				inside: {
					'punctuation': /:/,
					'variable': variable,
					'operator': operator
				}
			},
			// We want to consume the whole line
			'property-line': {
				pattern: /^[ \t]*(?:[^:\s]+ *:.*|:[^:\s].*)/m,
				inside: {
					'property': [
						/[^:\s]+(?=\s*:)/,
						{
							pattern: /(:)[^:\s]+/,
							lookbehind: true
						}
					],
					'punctuation': /:/,
					'variable': variable,
					'operator': operator,
					'important': Prism.languages.sass.important
				}
			}
		});
		delete Prism.languages.sass.property;
		delete Prism.languages.sass.important;

		// Now that whole lines for other patterns are consumed,
		// what's left should be selectors
		Prism.languages.insertBefore('sass', 'punctuation', {
			'selector': {
				pattern: /([ \t]*)\S(?:,[^,\r\n]+|[^,\r\n]*)(?:,[^,\r\n]+)*(?:,(?:\r?\n|\r)\1[ \t]+\S(?:,[^,\r\n]+|[^,\r\n]*)(?:,[^,\r\n]+)*)*/,
				lookbehind: true
			}
		});

	}(Prism));

	Prism.languages.scss = Prism.languages.extend('css', {
		'comment': {
			pattern: /(^|[^\\])(?:\/\*[\s\S]*?\*\/|\/\/.*)/,
			lookbehind: true
		},
		'atrule': {
			pattern: /@[\w-](?:\([^()]+\)|[^()\s]|\s+(?!\s))*?(?=\s+[{;])/,
			inside: {
				'rule': /@[\w-]+/
				// See rest below
			}
		},
		// url, compassified
		'url': /(?:[-a-z]+-)?url(?=\()/i,
		// CSS selector regex is not appropriate for Sass
		// since there can be lot more things (var, @ directive, nesting..)
		// a selector must start at the end of a property or after a brace (end of other rules or nesting)
		// it can contain some characters that aren't used for defining rules or end of selector, & (parent selector), or interpolated variable
		// the end of a selector is found when there is no rules in it ( {} or {\s}) or if there is a property (because an interpolated var
		// can "pass" as a selector- e.g: proper#{$erty})
		// this one was hard to do, so please be careful if you edit this one :)
		'selector': {
			// Initial look-ahead is used to prevent matching of blank selectors
			pattern: /(?=\S)[^@;{}()]?(?:[^@;{}()\s]|\s+(?!\s)|#\{\$[-\w]+\})+(?=\s*\{(?:\}|\s|[^}][^:{}]*[:{][^}]+))/m,
			inside: {
				'parent': {
					pattern: /&/,
					alias: 'important'
				},
				'placeholder': /%[-\w]+/,
				'variable': /\$[-\w]+|#\{\$[-\w]+\}/
			}
		},
		'property': {
			pattern: /(?:[-\w]|\$[-\w]|#\{\$[-\w]+\})+(?=\s*:)/,
			inside: {
				'variable': /\$[-\w]+|#\{\$[-\w]+\}/
			}
		}
	});

	Prism.languages.insertBefore('scss', 'atrule', {
		'keyword': [
			/@(?:if|else(?: if)?|forward|for|each|while|import|use|extend|debug|warn|mixin|include|function|return|content)\b/i,
			{
				pattern: /( +)(?:from|through)(?= )/,
				lookbehind: true
			}
		]
	});

	Prism.languages.insertBefore('scss', 'important', {
		// var and interpolated vars
		'variable': /\$[-\w]+|#\{\$[-\w]+\}/
	});

	Prism.languages.insertBefore('scss', 'function', {
		'module-modifier': {
			pattern: /\b(?:as|with|show|hide)\b/i,
			alias: 'keyword'
		},
		'placeholder': {
			pattern: /%[-\w]+/,
			alias: 'selector'
		},
		'statement': {
			pattern: /\B!(?:default|optional)\b/i,
			alias: 'keyword'
		},
		'boolean': /\b(?:true|false)\b/,
		'null': {
			pattern: /\bnull\b/,
			alias: 'keyword'
		},
		'operator': {
			pattern: /(\s)(?:[-+*\/%]|[=!]=|<=?|>=?|and|or|not)(?=\s)/,
			lookbehind: true
		}
	});

	Prism.languages.scss['atrule'].inside.rest = Prism.languages.scss;

	Prism.languages.scala = Prism.languages.extend('java', {
		'triple-quoted-string': {
			pattern: /"""[\s\S]*?"""/,
			greedy: true,
			alias: 'string'
		},
		'string': {
			pattern: /("|')(?:\\.|(?!\1)[^\\\r\n])*\1/,
			greedy: true
		},
		'keyword': /<-|=>|\b(?:abstract|case|catch|class|def|do|else|extends|final|finally|for|forSome|if|implicit|import|lazy|match|new|null|object|override|package|private|protected|return|sealed|self|super|this|throw|trait|try|type|val|var|while|with|yield)\b/,
		'number': /\b0x(?:[\da-f]*\.)?[\da-f]+|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e\d+)?[dfl]?/i,
		'builtin': /\b(?:String|Int|Long|Short|Byte|Boolean|Double|Float|Char|Any|AnyRef|AnyVal|Unit|Nothing)\b/,
		'symbol': /'[^\d\s\\]\w*/
	});
	delete Prism.languages.scala['class-name'];
	delete Prism.languages.scala['function'];

	(function (Prism) {
		Prism.languages.scheme = {
			// this supports "normal" single-line comments:
			//   ; comment
			// and (potentially nested) multiline comments:
			//   #| comment #| nested |# still comment |#
			// (only 1 level of nesting is supported)
			'comment': /;.*|#;\s*(?:\((?:[^()]|\([^()]*\))*\)|\[(?:[^\[\]]|\[[^\[\]]*\])*\])|#\|(?:[^#|]|#(?!\|)|\|(?!#)|#\|(?:[^#|]|#(?!\|)|\|(?!#))*\|#)*\|#/,
			'string': {
				pattern: /"(?:[^"\\]|\\.)*"/,
				greedy: true
			},
			'symbol': {
				pattern: /'[^()\[\]#'\s]+/,
				greedy: true
			},
			'character': {
				pattern: /#\\(?:[ux][a-fA-F\d]+\b|[-a-zA-Z]+\b|[\uD800-\uDBFF][\uDC00-\uDFFF]|\S)/,
				greedy: true,
				alias: 'string'
			},
			'lambda-parameter': [
				// https://www.cs.cmu.edu/Groups/AI/html/r4rs/r4rs_6.html#SEC30
				{
					pattern: /((?:^|[^'`#])[(\[]lambda\s+)(?:[^|()\[\]'\s]+|\|(?:[^\\|]|\\.)*\|)/,
					lookbehind: true
				},
				{
					pattern: /((?:^|[^'`#])[(\[]lambda\s+[(\[])[^()\[\]']+/,
					lookbehind: true
				}
			],
			'keyword': {
				pattern: /((?:^|[^'`#])[(\[])(?:begin|case(?:-lambda)?|cond(?:-expand)?|define(?:-library|-macro|-record-type|-syntax|-values)?|defmacro|delay(?:-force)?|do|else|export|except|guard|if|import|include(?:-ci|-library-declarations)?|lambda|let(?:rec)?(?:-syntax|-values|\*)?|let\*-values|only|parameterize|prefix|(?:quasi-?)?quote|rename|set!|syntax-(?:case|rules)|unless|unquote(?:-splicing)?|when)(?=[()\[\]\s]|$)/,
				lookbehind: true
			},
			'builtin': {
				// all functions of the base library of R7RS plus some of built-ins of R5Rs
				pattern: /((?:^|[^'`#])[(\[])(?:abs|and|append|apply|assoc|ass[qv]|binary-port\?|boolean=?\?|bytevector(?:-append|-copy|-copy!|-length|-u8-ref|-u8-set!|\?)?|caar|cadr|call-with-(?:current-continuation|port|values)|call\/cc|car|cdar|cddr|cdr|ceiling|char(?:->integer|-ready\?|\?|<\?|<=\?|=\?|>\?|>=\?)|close-(?:input-port|output-port|port)|complex\?|cons|current-(?:error|input|output)-port|denominator|dynamic-wind|eof-object\??|eq\?|equal\?|eqv\?|error|error-object(?:-irritants|-message|\?)|eval|even\?|exact(?:-integer-sqrt|-integer\?|\?)?|expt|features|file-error\?|floor(?:-quotient|-remainder|\/)?|flush-output-port|for-each|gcd|get-output-(?:bytevector|string)|inexact\??|input-port(?:-open\?|\?)|integer(?:->char|\?)|lcm|length|list(?:->string|->vector|-copy|-ref|-set!|-tail|\?)?|make-(?:bytevector|list|parameter|string|vector)|map|max|member|memq|memv|min|modulo|negative\?|newline|not|null\?|number(?:->string|\?)|numerator|odd\?|open-(?:input|output)-(?:bytevector|string)|or|output-port(?:-open\?|\?)|pair\?|peek-char|peek-u8|port\?|positive\?|procedure\?|quotient|raise|raise-continuable|rational\?|rationalize|read-(?:bytevector|bytevector!|char|error\?|line|string|u8)|real\?|remainder|reverse|round|set-c[ad]r!|square|string(?:->list|->number|->symbol|->utf8|->vector|-append|-copy|-copy!|-fill!|-for-each|-length|-map|-ref|-set!|\?|<\?|<=\?|=\?|>\?|>=\?)?|substring|symbol(?:->string|\?|=\?)|syntax-error|textual-port\?|truncate(?:-quotient|-remainder|\/)?|u8-ready\?|utf8->string|values|vector(?:->list|->string|-append|-copy|-copy!|-fill!|-for-each|-length|-map|-ref|-set!|\?)?|with-exception-handler|write-(?:bytevector|char|string|u8)|zero\?)(?=[()\[\]\s]|$)/,
				lookbehind: true
			},
			'operator': {
				pattern: /((?:^|[^'`#])[(\[])(?:[-+*%/]|[<>]=?|=>?)(?=[()\[\]\s]|$)/,
				lookbehind: true
			},
			'number': {
				// The number pattern from [the R7RS spec](https://small.r7rs.org/attachment/r7rs.pdf).
				//
				// <number>      := <num 2>|<num 8>|<num 10>|<num 16>
				// <num R>       := <prefix R><complex R>
				// <complex R>   := <real R>(?:@<real R>|<imaginary R>)?|<imaginary R>
				// <imaginary R> := [+-](?:<ureal R>|(?:inf|nan)\.0)?i
				// <real R>      := [+-]?<ureal R>|[+-](?:inf|nan)\.0
				// <ureal R>     := <uint R>(?:\/<uint R>)?
				//                | <decimal R>
				//
				// <decimal 10>  := (?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?
				// <uint R>      := <digit R>+
				// <prefix R>    := <radix R>(?:#[ei])?|(?:#[ei])?<radix R>
				// <radix 2>     := #b
				// <radix 8>     := #o
				// <radix 10>    := (?:#d)?
				// <radix 16>    := #x
				// <digit 2>     := [01]
				// <digit 8>     := [0-7]
				// <digit 10>    := \d
				// <digit 16>    := [0-9a-f]
				//
				// The problem with this grammar is that the resulting regex is way to complex, so we simplify by grouping all
				// non-decimal bases together. This results in a decimal (dec) and combined binary, octal, and hexadecimal (box)
				// pattern:
				pattern: RegExp(SortedBNF({
					'<ureal dec>': /\d+(?:\/\d+)?|(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?/.source,
					'<real dec>': /[+-]?<ureal dec>|[+-](?:inf|nan)\.0/.source,
					'<imaginary dec>': /[+-](?:<ureal dec>|(?:inf|nan)\.0)?i/.source,
					'<complex dec>': /<real dec>(?:@<real dec>|<imaginary dec>)?|<imaginary dec>/.source,
					'<num dec>': /(?:#d(?:#[ei])?|#[ei](?:#d)?)?<complex dec>/.source,

					'<ureal box>': /[0-9a-f]+(?:\/[0-9a-f]+)?/.source,
					'<real box>': /[+-]?<ureal box>|[+-](?:inf|nan)\.0/.source,
					'<imaginary box>': /[+-](?:<ureal box>|(?:inf|nan)\.0)?i/.source,
					'<complex box>': /<real box>(?:@<real box>|<imaginary box>)?|<imaginary box>/.source,
					'<num box>': /#[box](?:#[ei])?|(?:#[ei])?#[box]<complex box>/.source,

					'<number>': /(^|[()\[\]\s])(?:<num dec>|<num box>)(?=[()\[\]\s]|$)/.source,
				}), 'i'),
				lookbehind: true
			},
			'boolean': {
				pattern: /(^|[()\[\]\s])#(?:[ft]|false|true)(?=[()\[\]\s]|$)/,
				lookbehind: true
			},
			'function': {
				pattern: /((?:^|[^'`#])[(\[])(?:[^|()\[\]'\s]+|\|(?:[^\\|]|\\.)*\|)(?=[()\[\]\s]|$)/,
				lookbehind: true
			},
			'identifier': {
				pattern: /(^|[()\[\]\s])\|(?:[^\\|]|\\.)*\|(?=[()\[\]\s]|$)/,
				lookbehind: true,
				greedy: true
			},
			'punctuation': /[()\[\]']/
		};

		/**
		 * Given a topologically sorted BNF grammar, this will return the RegExp source of last rule of the grammar.
		 *
		 * @param {Record<string, string>} grammar
		 * @returns {string}
		 */
		function SortedBNF(grammar) {
			for (var key in grammar) {
				grammar[key] = grammar[key].replace(/<[\w\s]+>/g, function (key) {
					return '(?:' + grammar[key].trim() + ')';
				});
			}
			// return the last item
			return grammar[key];
		}

	}(Prism));

	Prism.languages.smalltalk = {
		'comment': /"(?:""|[^"])*"/,
		'character': {
			pattern: /\$./,
			alias: 'string'
		},
		'string': /'(?:''|[^'])*'/,
		'symbol': /#[\da-z]+|#(?:-|([+\/\\*~<>=@%|&?!])\1?)|#(?=\()/i,
		'block-arguments': {
			pattern: /(\[\s*):[^\[|]*\|/,
			lookbehind: true,
			inside: {
				'variable': /:[\da-z]+/i,
				'punctuation': /\|/
			}
		},
		'temporary-variables': {
			pattern: /\|[^|]+\|/,
			inside: {
				'variable': /[\da-z]+/i,
				'punctuation': /\|/
			}
		},
		'keyword': /\b(?:nil|true|false|self|super|new)\b/,
		'number': [
			/\d+r-?[\dA-Z]+(?:\.[\dA-Z]+)?(?:e-?\d+)?/,
			/\b\d+(?:\.\d+)?(?:e-?\d+)?/
		],
		'operator': /[<=]=?|:=|~[~=]|\/\/?|\\\\|>[>=]?|[!^+\-*&|,@]/,
		'punctuation': /[.;:?\[\](){}]/
	};

	/* TODO
		Add support for variables inside double quoted strings
		Add support for {php}
	*/

	(function (Prism) {

		Prism.languages.smarty = {
			'comment': /\{\*[\s\S]*?\*\}/,
			'delimiter': {
				pattern: /^\{|\}$/i,
				alias: 'punctuation'
			},
			'string': /(["'])(?:\\.|(?!\1)[^\\\r\n])*\1/,
			'number': /\b0x[\dA-Fa-f]+|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[Ee][-+]?\d+)?/,
			'variable': [
				/\$(?!\d)\w+/,
				/#(?!\d)\w+#/,
				{
					pattern: /(\.|->)(?!\d)\w+/,
					lookbehind: true
				},
				{
					pattern: /(\[)(?!\d)\w+(?=\])/,
					lookbehind: true
				}
			],
			'function': [
				{
					pattern: /(\|\s*)@?(?!\d)\w+/,
					lookbehind: true
				},
				/^\/?(?!\d)\w+/,
				/(?!\d)\w+(?=\()/
			],
			'attr-name': {
				// Value is made optional because it may have already been tokenized
				pattern: /\w+\s*=\s*(?:(?!\d)\w+)?/,
				inside: {
					'variable': {
						pattern: /(=\s*)(?!\d)\w+/,
						lookbehind: true
					},
					'operator': /=/
				}
			},
			'punctuation': [
				/[\[\]().,:`]|->/
			],
			'operator': [
				/[+\-*\/%]|==?=?|[!<>]=?|&&|\|\|?/,
				/\bis\s+(?:not\s+)?(?:div|even|odd)(?:\s+by)?\b/,
				/\b(?:eq|neq?|gt|lt|gt?e|lt?e|not|mod|or|and)\b/
			],
			'keyword': /\b(?:false|off|on|no|true|yes)\b/
		};

		// Tokenize all inline Smarty expressions
		Prism.hooks.add('before-tokenize', function (env) {
			var smartyPattern = /\{\*[\s\S]*?\*\}|\{[\s\S]+?\}/g;
			var smartyLitteralStart = '{literal}';
			var smartyLitteralEnd = '{/literal}';
			var smartyLitteralMode = false;

			Prism.languages['markup-templating'].buildPlaceholders(env, 'smarty', smartyPattern, function (match) {
				// Smarty tags inside {literal} block are ignored
				if (match === smartyLitteralEnd) {
					smartyLitteralMode = false;
				}

				if (!smartyLitteralMode) {
					if (match === smartyLitteralStart) {
						smartyLitteralMode = true;
					}

					return true;
				}
				return false;
			});
		});

		// Re-insert the tokens after tokenizing
		Prism.hooks.add('after-tokenize', function (env) {
			Prism.languages['markup-templating'].tokenizePlaceholders(env, 'smarty');
		});

	}(Prism));

	(function (Prism) {
		var unit = {
			pattern: /(\b\d+)(?:%|[a-z]+)/,
			lookbehind: true
		};
		// 123 -123 .123 -.123 12.3 -12.3
		var number = {
			pattern: /(^|[^\w.-])-?(?:\d+(?:\.\d+)?|\.\d+)/,
			lookbehind: true
		};

		var inside = {
			'comment': {
				pattern: /(^|[^\\])(?:\/\*[\s\S]*?\*\/|\/\/.*)/,
				lookbehind: true
			},
			'url': {
				pattern: /url\((["']?).*?\1\)/i,
				greedy: true
			},
			'string': {
				pattern: /("|')(?:(?!\1)[^\\\r\n]|\\(?:\r\n|[\s\S]))*\1/,
				greedy: true
			},
			'interpolation': null, // See below
			'func': null, // See below
			'important': /\B!(?:important|optional)\b/i,
			'keyword': {
				pattern: /(^|\s+)(?:(?:if|else|for|return|unless)(?=\s+|$)|@[\w-]+)/,
				lookbehind: true
			},
			'hexcode': /#[\da-f]{3,6}/i,
			'color': [
				/\b(?:AliceBlue|AntiqueWhite|Aqua|Aquamarine|Azure|Beige|Bisque|Black|BlanchedAlmond|Blue|BlueViolet|Brown|BurlyWood|CadetBlue|Chartreuse|Chocolate|Coral|CornflowerBlue|Cornsilk|Crimson|Cyan|DarkBlue|DarkCyan|DarkGoldenRod|DarkGr[ae]y|DarkGreen|DarkKhaki|DarkMagenta|DarkOliveGreen|DarkOrange|DarkOrchid|DarkRed|DarkSalmon|DarkSeaGreen|DarkSlateBlue|DarkSlateGr[ae]y|DarkTurquoise|DarkViolet|DeepPink|DeepSkyBlue|DimGr[ae]y|DodgerBlue|FireBrick|FloralWhite|ForestGreen|Fuchsia|Gainsboro|GhostWhite|Gold|GoldenRod|Gr[ae]y|Green|GreenYellow|HoneyDew|HotPink|IndianRed|Indigo|Ivory|Khaki|Lavender|LavenderBlush|LawnGreen|LemonChiffon|LightBlue|LightCoral|LightCyan|LightGoldenRodYellow|LightGr[ae]y|LightGreen|LightPink|LightSalmon|LightSeaGreen|LightSkyBlue|LightSlateGr[ae]y|LightSteelBlue|LightYellow|Lime|LimeGreen|Linen|Magenta|Maroon|MediumAquaMarine|MediumBlue|MediumOrchid|MediumPurple|MediumSeaGreen|MediumSlateBlue|MediumSpringGreen|MediumTurquoise|MediumVioletRed|MidnightBlue|MintCream|MistyRose|Moccasin|NavajoWhite|Navy|OldLace|Olive|OliveDrab|Orange|OrangeRed|Orchid|PaleGoldenRod|PaleGreen|PaleTurquoise|PaleVioletRed|PapayaWhip|PeachPuff|Peru|Pink|Plum|PowderBlue|Purple|Red|RosyBrown|RoyalBlue|SaddleBrown|Salmon|SandyBrown|SeaGreen|SeaShell|Sienna|Silver|SkyBlue|SlateBlue|SlateGr[ae]y|Snow|SpringGreen|SteelBlue|Tan|Teal|Thistle|Tomato|Transparent|Turquoise|Violet|Wheat|White|WhiteSmoke|Yellow|YellowGreen)\b/i,
				{
					pattern: /\b(?:rgb|hsl)\(\s*\d{1,3}\s*,\s*\d{1,3}%?\s*,\s*\d{1,3}%?\s*\)\B|\b(?:rgb|hsl)a\(\s*\d{1,3}\s*,\s*\d{1,3}%?\s*,\s*\d{1,3}%?\s*,\s*(?:0|0?\.\d+|1)\s*\)\B/i,
					inside: {
						'unit': unit,
						'number': number,
						'function': /[\w-]+(?=\()/,
						'punctuation': /[(),]/
					}
				}
			],
			'entity': /\\[\da-f]{1,8}/i,
			'unit': unit,
			'boolean': /\b(?:true|false)\b/,
			'operator': [
				// We want non-word chars around "-" because it is
				// accepted in property names.
				/~|[+!\/%<>?=]=?|[-:]=|\*[*=]?|\.{2,3}|&&|\|\||\B-\B|\b(?:and|in|is(?: a| defined| not|nt)?|not|or)\b/
			],
			'number': number,
			'punctuation': /[{}()\[\];:,]/
		};

		inside['interpolation'] = {
			pattern: /\{[^\r\n}:]+\}/,
			alias: 'variable',
			inside: {
				'delimiter': {
					pattern: /^{|}$/,
					alias: 'punctuation'
				},
				rest: inside
			}
		};
		inside['func'] = {
			pattern: /[\w-]+\([^)]*\).*/,
			inside: {
				'function': /^[^(]+/,
				rest: inside
			}
		};

		Prism.languages.stylus = {
			'atrule-declaration': {
				pattern: /(^\s*)@.+/m,
				lookbehind: true,
				inside: {
					'atrule': /^@[\w-]+/,
					rest: inside
				}
			},
			'variable-declaration': {
				pattern: /(^[ \t]*)[\w$-]+\s*.?=[ \t]*(?:\{[^{}]*\}|\S.*|$)/m,
				lookbehind: true,
				inside: {
					'variable': /^\S+/,
					rest: inside
				}
			},

			'statement': {
				pattern: /(^[ \t]*)(?:if|else|for|return|unless)[ \t].+/m,
				lookbehind: true,
				inside: {
					'keyword': /^\S+/,
					rest: inside
				}
			},

			// A property/value pair cannot end with a comma or a brace
			// It cannot have indented content unless it ended with a semicolon
			'property-declaration': {
				pattern: /((?:^|\{)([ \t]*))(?:[\w-]|\{[^}\r\n]+\})+(?:\s*:\s*|[ \t]+)(?!\s)[^{\r\n]*(?:;|[^{\r\n,](?=$)(?!(?:\r?\n|\r)(?:\{|\2[ \t]+)))/m,
				lookbehind: true,
				inside: {
					'property': {
						pattern: /^[^\s:]+/,
						inside: {
							'interpolation': inside.interpolation
						}
					},
					rest: inside
				}
			},



			// A selector can contain parentheses only as part of a pseudo-element
			// It can span multiple lines.
			// It must end with a comma or an accolade or have indented content.
			'selector': {
				pattern: /(^[ \t]*)(?:(?=\S)(?:[^{}\r\n:()]|::?[\w-]+(?:\([^)\r\n]*\)|(?![\w-]))|\{[^}\r\n]+\})+)(?:(?:\r?\n|\r)(?:\1(?:(?=\S)(?:[^{}\r\n:()]|::?[\w-]+(?:\([^)\r\n]*\)|(?![\w-]))|\{[^}\r\n]+\})+)))*(?:,$|\{|(?=(?:\r?\n|\r)(?:\{|\1[ \t]+)))/m,
				lookbehind: true,
				inside: {
					'interpolation': inside.interpolation,
					'comment': inside.comment,
					'punctuation': /[{},]/
				}
			},

			'func': inside.func,
			'string': inside.string,
			'comment': {
				pattern: /(^|[^\\])(?:\/\*[\s\S]*?\*\/|\/\/.*)/,
				lookbehind: true,
				greedy: true
			},
			'interpolation': inside.interpolation,
			'punctuation': /[{}()\[\];:.]/
		};
	}(Prism));

	// issues: nested multiline comments
	Prism.languages.swift = Prism.languages.extend('clike', {
		'string': {
			pattern: /("|')(?:\\(?:\((?:[^()]|\([^)]+\))+\)|\r\n|[^(])|(?!\1)[^\\\r\n])*\1/,
			greedy: true,
			inside: {
				'interpolation': {
					pattern: /\\\((?:[^()]|\([^)]+\))+\)/,
					inside: {
						delimiter: {
							pattern: /^\\\(|\)$/,
							alias: 'variable'
						}
						// See rest below
					}
				}
			}
		},
		'keyword': /\b(?:as|associativity|break|case|catch|class|continue|convenience|default|defer|deinit|didSet|do|dynamic(?:Type)?|else|enum|extension|fallthrough|final|for|func|get|guard|if|import|in|infix|init|inout|internal|is|lazy|left|let|mutating|new|none|nonmutating|operator|optional|override|postfix|precedence|prefix|private|protocol|public|repeat|required|rethrows|return|right|safe|self|Self|set|some|static|struct|subscript|super|switch|throws?|try|Type|typealias|unowned|unsafe|var|weak|where|while|willSet|__(?:COLUMN__|FILE__|FUNCTION__|LINE__))\b/,
		'number': /\b(?:[\d_]+(?:\.[\de_]+)?|0x[a-f0-9_]+(?:\.[a-f0-9p_]+)?|0b[01_]+|0o[0-7_]+)\b/i,
		'constant': /\b(?:nil|[A-Z_]{2,}|k[A-Z][A-Za-z_]+)\b/,
		'atrule': /@\b(?:IB(?:Outlet|Designable|Action|Inspectable)|class_protocol|exported|noreturn|NS(?:Copying|Managed)|objc|UIApplicationMain|auto_closure)\b/,
		'builtin': /\b(?:[A-Z]\S+|abs|advance|alignof(?:Value)?|assert|contains|count(?:Elements)?|debugPrint(?:ln)?|distance|drop(?:First|Last)|dump|enumerate|equal|filter|find|first|getVaList|indices|isEmpty|join|last|lexicographicalCompare|map|max(?:Element)?|min(?:Element)?|numericCast|overlaps|partition|print(?:ln)?|reduce|reflect|reverse|sizeof(?:Value)?|sort(?:ed)?|split|startsWith|stride(?:of(?:Value)?)?|suffix|swap|toDebugString|toString|transcode|underestimateCount|unsafeBitCast|with(?:ExtendedLifetime|Unsafe(?:MutablePointers?|Pointers?)|VaList))\b/
	});
	Prism.languages.swift['string'].inside['interpolation'].inside.rest = Prism.languages.swift;

	(function (Prism) {

		var key = /(?:[\w-]+|'[^'\n\r]*'|"(?:\\.|[^\\"\r\n])*")/.source;

		/**
		 * @param {string} pattern
		 */
		function insertKey(pattern) {
			return pattern.replace(/__/g, function () { return key; });
		}

		Prism.languages.toml = {
			'comment': {
				pattern: /#.*/,
				greedy: true
			},
			'table': {
				pattern: RegExp(insertKey(/(^\s*\[\s*(?:\[\s*)?)__(?:\s*\.\s*__)*(?=\s*\])/.source), 'm'),
				lookbehind: true,
				greedy: true,
				alias: 'class-name'
			},
			'key': {
				pattern: RegExp(insertKey(/(^\s*|[{,]\s*)__(?:\s*\.\s*__)*(?=\s*=)/.source), 'm'),
				lookbehind: true,
				greedy: true,
				alias: 'property'
			},
			'string': {
				pattern: /"""(?:\\[\s\S]|[^\\])*?"""|'''[\s\S]*?'''|'[^'\n\r]*'|"(?:\\.|[^\\"\r\n])*"/,
				greedy: true
			},
			'date': [
				{
					// Offset Date-Time, Local Date-Time, Local Date
					pattern: /\b\d{4}-\d{2}-\d{2}(?:[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)?\b/i,
					alias: 'number'
				},
				{
					// Local Time
					pattern: /\b\d{2}:\d{2}:\d{2}(?:\.\d+)?\b/,
					alias: 'number'
				}
			],
			'number': /(?:\b0(?:x[\da-zA-Z]+(?:_[\da-zA-Z]+)*|o[0-7]+(?:_[0-7]+)*|b[10]+(?:_[10]+)*))\b|[-+]?\b\d+(?:_\d+)*(?:\.\d+(?:_\d+)*)?(?:[eE][+-]?\d+(?:_\d+)*)?\b|[-+]?\b(?:inf|nan)\b/,
			'boolean': /\b(?:true|false)\b/,
			'punctuation': /[.,=[\]{}]/
		};
	}(Prism));

	(function (Prism) {
		var interpolationExpr = {
			pattern: /[\s\S]+/,
			inside: null
		};

		Prism.languages.v = Prism.languages.extend('clike', {
			'string': [
				{
					pattern: /`(?:\\\`|\\?[^\`]{1,2})`/, // using {1,2} instead of `u` flag for compatibility
					alias: 'rune'
				},
				{
					pattern: /r?(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/,
					alias: 'quoted-string',
					greedy: true,
					inside: {
						'interpolation': {
							pattern: /((?:^|[^\\])(?:\\{2})*)\$(?:\{[^{}]*\}|\w+(?:\.\w+(?:\([^\(\)]*\))?|\[[^\[\]]+\])*)/,
							lookbehind: true,
							inside: {
								'interpolation-variable': {
									pattern: /^\$\w[\s\S]*$/,
									alias: 'variable'
								},
								'interpolation-punctuation': {
									pattern: /^\${|}$/,
									alias: 'punctuation'
								},
								'interpolation-expression': interpolationExpr
							}
						}
					}
				}
			],
			'class-name': {
				pattern: /(\b(?:enum|interface|struct|type)\s+)(?:C\.)?[\w]+/,
				lookbehind: true
			},
			'keyword': /(?:\b(?:as|asm|assert|atomic|break|chan|const|continue|defer|else|embed|enum|fn|for|__global|go(?:to)?|if|import|in|interface|is|lock|match|module|mut|none|or|pub|return|rlock|select|shared|sizeof|static|struct|type(?:of)?|union|unsafe)|\$(?:if|else|for)|#(?:include|flag))\b/,
			'number': /\b(?:0x[a-f\d]+(?:_[a-f\d]+)*|0b[01]+(?:_[01]+)*|0o[0-7]+(?:_[0-7]+)*|\d+(?:_\d+)*(?:\.\d+(?:_\d+)*)?)\b/i,
			'operator': /~|\?|[*\/%^!=]=?|\+[=+]?|-[=-]?|\|[=|]?|&(?:=|&|\^=?)?|>(?:>=?|=)?|<(?:<=?|=|-)?|:=|\.\.\.?/,
			'builtin': /\b(?:any(?:_int|_float)?|bool|byte(?:ptr)?|charptr|f(?:32|64)|i(?:8|16|nt|64|128)|rune|size_t|string|u(?:16|32|64|128)|voidptr)\b/
		});

		interpolationExpr.inside = Prism.languages.v;

		Prism.languages.insertBefore('v', 'operator', {
			'attribute': {
				pattern: /^\s*\[(?:deprecated|unsafe_fn|typedef|live|inline|flag|ref_only|windows_stdcall|direct_array_access)\]/m,
				alias: 'annotation',
				inside: {
					'punctuation': /[\[\]]/,
					'keyword': /\w+/
				}
			},
			'generic': {
				pattern: /\<\w+\>(?=\s*[\)\{])/,
				inside: {
					'punctuation': /[<>]/,
					'class-name': /\w+/
				}
			}
		});

		Prism.languages.insertBefore('v', 'function', {
			'generic-function': {
				// e.g. foo<T>( ...
				pattern: /\w+\s*<\w+>(?=\()/,
				inside: {
					'function': /^\w+/,
					'generic': {
						pattern: /<\w+>/,
						inside: Prism.languages.v.generic.inside
					}
				}
			}
		});
	}(Prism));
	Prism.languages.vim = {
		'string': /"(?:[^"\\\r\n]|\\.)*"|'(?:[^'\r\n]|'')*'/,
		'comment': /".*/,
		'function': /\w+(?=\()/,
		'keyword': /\b(?:ab|abbreviate|abc|abclear|abo|aboveleft|al|all|arga|argadd|argd|argdelete|argdo|arge|argedit|argg|argglobal|argl|arglocal|ar|args|argu|argument|as|ascii|bad|badd|ba|ball|bd|bdelete|be|bel|belowright|bf|bfirst|bl|blast|bm|bmodified|bn|bnext|bN|bNext|bo|botright|bp|bprevious|brea|break|breaka|breakadd|breakd|breakdel|breakl|breaklist|br|brewind|bro|browse|bufdo|b|buffer|buffers|bun|bunload|bw|bwipeout|ca|cabbrev|cabc|cabclear|caddb|caddbuffer|cad|caddexpr|caddf|caddfile|cal|call|cat|catch|cb|cbuffer|cc|ccl|cclose|cd|ce|center|cex|cexpr|cf|cfile|cfir|cfirst|cgetb|cgetbuffer|cgete|cgetexpr|cg|cgetfile|c|change|changes|chd|chdir|che|checkpath|checkt|checktime|cla|clast|cl|clist|clo|close|cmapc|cmapclear|cnew|cnewer|cn|cnext|cN|cNext|cnf|cnfile|cNfcNfile|cnorea|cnoreabbrev|col|colder|colo|colorscheme|comc|comclear|comp|compiler|conf|confirm|con|continue|cope|copen|co|copy|cpf|cpfile|cp|cprevious|cq|cquit|cr|crewind|cuna|cunabbrev|cu|cunmap|cw|cwindow|debugg|debuggreedy|delc|delcommand|d|delete|delf|delfunction|delm|delmarks|diffg|diffget|diffoff|diffpatch|diffpu|diffput|diffsplit|diffthis|diffu|diffupdate|dig|digraphs|di|display|dj|djump|dl|dlist|dr|drop|ds|dsearch|dsp|dsplit|earlier|echoe|echoerr|echom|echomsg|echon|e|edit|el|else|elsei|elseif|em|emenu|endfo|endfor|endf|endfunction|endfun|en|endif|endt|endtry|endw|endwhile|ene|enew|ex|exi|exit|exu|exusage|f|file|files|filetype|fina|finally|fin|find|fini|finish|fir|first|fix|fixdel|fo|fold|foldc|foldclose|folddoc|folddoclosed|foldd|folddoopen|foldo|foldopen|for|fu|fun|function|go|goto|gr|grep|grepa|grepadd|ha|hardcopy|h|help|helpf|helpfind|helpg|helpgrep|helpt|helptags|hid|hide|his|history|ia|iabbrev|iabc|iabclear|if|ij|ijump|il|ilist|imapc|imapclear|in|inorea|inoreabbrev|isearch|isp|isplit|iuna|iunabbrev|iu|iunmap|j|join|ju|jumps|k|keepalt|keepj|keepjumps|kee|keepmarks|laddb|laddbuffer|lad|laddexpr|laddf|laddfile|lan|language|la|last|later|lb|lbuffer|lc|lcd|lch|lchdir|lcl|lclose|let|left|lefta|leftabove|lex|lexpr|lf|lfile|lfir|lfirst|lgetb|lgetbuffer|lgete|lgetexpr|lg|lgetfile|lgr|lgrep|lgrepa|lgrepadd|lh|lhelpgrep|l|list|ll|lla|llast|lli|llist|lmak|lmake|lm|lmap|lmapc|lmapclear|lnew|lnewer|lne|lnext|lN|lNext|lnf|lnfile|lNf|lNfile|ln|lnoremap|lo|loadview|loc|lockmarks|lockv|lockvar|lol|lolder|lop|lopen|lpf|lpfile|lp|lprevious|lr|lrewind|ls|lt|ltag|lu|lunmap|lv|lvimgrep|lvimgrepa|lvimgrepadd|lw|lwindow|mak|make|ma|mark|marks|mat|match|menut|menutranslate|mk|mkexrc|mks|mksession|mksp|mkspell|mkvie|mkview|mkv|mkvimrc|mod|mode|m|move|mzf|mzfile|mz|mzscheme|nbkey|new|n|next|N|Next|nmapc|nmapclear|noh|nohlsearch|norea|noreabbrev|nu|number|nun|nunmap|omapc|omapclear|on|only|o|open|opt|options|ou|ounmap|pc|pclose|ped|pedit|pe|perl|perld|perldo|po|pop|popu|popup|pp|ppop|pre|preserve|prev|previous|p|print|P|Print|profd|profdel|prof|profile|promptf|promptfind|promptr|promptrepl|ps|psearch|pta|ptag|ptf|ptfirst|ptj|ptjump|ptl|ptlast|ptn|ptnext|ptN|ptNext|ptp|ptprevious|ptr|ptrewind|pts|ptselect|pu|put|pw|pwd|pyf|pyfile|py|python|qa|qall|q|quit|quita|quitall|r|read|rec|recover|redi|redir|red|redo|redr|redraw|redraws|redrawstatus|reg|registers|res|resize|ret|retab|retu|return|rew|rewind|ri|right|rightb|rightbelow|rub|ruby|rubyd|rubydo|rubyf|rubyfile|ru|runtime|rv|rviminfo|sal|sall|san|sandbox|sa|sargument|sav|saveas|sba|sball|sbf|sbfirst|sbl|sblast|sbm|sbmodified|sbn|sbnext|sbN|sbNext|sbp|sbprevious|sbr|sbrewind|sb|sbuffer|scripte|scriptencoding|scrip|scriptnames|se|set|setf|setfiletype|setg|setglobal|setl|setlocal|sf|sfind|sfir|sfirst|sh|shell|sign|sil|silent|sim|simalt|sla|slast|sl|sleep|sm|smagic|smap|smapc|smapclear|sme|smenu|sn|snext|sN|sNext|sni|sniff|sno|snomagic|snor|snoremap|snoreme|snoremenu|sor|sort|so|source|spelld|spelldump|spe|spellgood|spelli|spellinfo|spellr|spellrepall|spellu|spellundo|spellw|spellwrong|sp|split|spr|sprevious|sre|srewind|sta|stag|startg|startgreplace|star|startinsert|startr|startreplace|stj|stjump|st|stop|stopi|stopinsert|sts|stselect|sun|sunhide|sunm|sunmap|sus|suspend|sv|sview|syncbind|t|tab|tabc|tabclose|tabd|tabdo|tabe|tabedit|tabf|tabfind|tabfir|tabfirst|tabl|tablast|tabm|tabmove|tabnew|tabn|tabnext|tabN|tabNext|tabo|tabonly|tabp|tabprevious|tabr|tabrewind|tabs|ta|tag|tags|tc|tcl|tcld|tcldo|tclf|tclfile|te|tearoff|tf|tfirst|th|throw|tj|tjump|tl|tlast|tm|tmenu|tn|tnext|tN|tNext|to|topleft|tp|tprevious|tr|trewind|try|ts|tselect|tu|tunmenu|una|unabbreviate|u|undo|undoj|undojoin|undol|undolist|unh|unhide|unlet|unlo|unlockvar|unm|unmap|up|update|verb|verbose|ve|version|vert|vertical|vie|view|vim|vimgrep|vimgrepa|vimgrepadd|vi|visual|viu|viusage|vmapc|vmapclear|vne|vnew|vs|vsplit|vu|vunmap|wa|wall|wh|while|winc|wincmd|windo|winp|winpos|win|winsize|wn|wnext|wN|wNext|wp|wprevious|wq|wqa|wqall|w|write|ws|wsverb|wv|wviminfo|X|xa|xall|x|xit|xm|xmap|xmapc|xmapclear|xme|xmenu|XMLent|XMLns|xn|xnoremap|xnoreme|xnoremenu|xu|xunmap|y|yank)\b/,
		'builtin': /\b(?:autocmd|acd|ai|akm|aleph|allowrevins|altkeymap|ambiwidth|ambw|anti|antialias|arab|arabic|arabicshape|ari|arshape|autochdir|autoindent|autoread|autowrite|autowriteall|aw|awa|background|backspace|backup|backupcopy|backupdir|backupext|backupskip|balloondelay|ballooneval|balloonexpr|bdir|bdlay|beval|bex|bexpr|bg|bh|bin|binary|biosk|bioskey|bk|bkc|bomb|breakat|brk|browsedir|bs|bsdir|bsk|bt|bufhidden|buflisted|buftype|casemap|ccv|cdpath|cedit|cfu|ch|charconvert|ci|cin|cindent|cink|cinkeys|cino|cinoptions|cinw|cinwords|clipboard|cmdheight|cmdwinheight|cmp|cms|columns|com|comments|commentstring|compatible|complete|completefunc|completeopt|consk|conskey|copyindent|cot|cpo|cpoptions|cpt|cscopepathcomp|cscopeprg|cscopequickfix|cscopetag|cscopetagorder|cscopeverbose|cspc|csprg|csqf|cst|csto|csverb|cuc|cul|cursorcolumn|cursorline|cwh|debug|deco|def|define|delcombine|dex|dg|dict|dictionary|diff|diffexpr|diffopt|digraph|dip|dir|directory|dy|ea|ead|eadirection|eb|ed|edcompatible|ef|efm|ei|ek|enc|encoding|endofline|eol|ep|equalalways|equalprg|errorbells|errorfile|errorformat|esckeys|et|eventignore|expandtab|exrc|fcl|fcs|fdc|fde|fdi|fdl|fdls|fdm|fdn|fdo|fdt|fen|fenc|fencs|fex|ff|ffs|fileencoding|fileencodings|fileformat|fileformats|fillchars|fk|fkmap|flp|fml|fmr|foldcolumn|foldenable|foldexpr|foldignore|foldlevel|foldlevelstart|foldmarker|foldmethod|foldminlines|foldnestmax|foldtext|formatexpr|formatlistpat|formatoptions|formatprg|fp|fs|fsync|ft|gcr|gd|gdefault|gfm|gfn|gfs|gfw|ghr|gp|grepformat|grepprg|gtl|gtt|guicursor|guifont|guifontset|guifontwide|guiheadroom|guioptions|guipty|guitablabel|guitabtooltip|helpfile|helpheight|helplang|hf|hh|hi|hidden|highlight|hk|hkmap|hkmapp|hkp|hl|hlg|hls|hlsearch|ic|icon|iconstring|ignorecase|im|imactivatekey|imak|imc|imcmdline|imd|imdisable|imi|iminsert|ims|imsearch|inc|include|includeexpr|incsearch|inde|indentexpr|indentkeys|indk|inex|inf|infercase|insertmode|isf|isfname|isi|isident|isk|iskeyword|isprint|joinspaces|js|key|keymap|keymodel|keywordprg|km|kmp|kp|langmap|langmenu|laststatus|lazyredraw|lbr|lcs|linebreak|lines|linespace|lisp|lispwords|listchars|loadplugins|lpl|lsp|lz|macatsui|magic|makeef|makeprg|matchpairs|matchtime|maxcombine|maxfuncdepth|maxmapdepth|maxmem|maxmempattern|maxmemtot|mco|mef|menuitems|mfd|mh|mis|mkspellmem|ml|mls|mm|mmd|mmp|mmt|modeline|modelines|modifiable|modified|more|mouse|mousef|mousefocus|mousehide|mousem|mousemodel|mouses|mouseshape|mouset|mousetime|mp|mps|msm|mzq|mzquantum|nf|nrformats|numberwidth|nuw|odev|oft|ofu|omnifunc|opendevice|operatorfunc|opfunc|osfiletype|pa|para|paragraphs|paste|pastetoggle|patchexpr|patchmode|path|pdev|penc|pex|pexpr|pfn|ph|pheader|pi|pm|pmbcs|pmbfn|popt|preserveindent|previewheight|previewwindow|printdevice|printencoding|printexpr|printfont|printheader|printmbcharset|printmbfont|printoptions|prompt|pt|pumheight|pvh|pvw|qe|quoteescape|readonly|remap|report|restorescreen|revins|rightleft|rightleftcmd|rl|rlc|ro|rs|rtp|ruf|ruler|rulerformat|runtimepath|sbo|sc|scb|scr|scroll|scrollbind|scrolljump|scrolloff|scrollopt|scs|sect|sections|secure|sel|selection|selectmode|sessionoptions|sft|shcf|shellcmdflag|shellpipe|shellquote|shellredir|shellslash|shelltemp|shelltype|shellxquote|shiftround|shiftwidth|shm|shortmess|shortname|showbreak|showcmd|showfulltag|showmatch|showmode|showtabline|shq|si|sidescroll|sidescrolloff|siso|sj|slm|smartcase|smartindent|smarttab|smc|smd|softtabstop|sol|spc|spell|spellcapcheck|spellfile|spelllang|spellsuggest|spf|spl|splitbelow|splitright|sps|sr|srr|ss|ssl|ssop|stal|startofline|statusline|stl|stmp|su|sua|suffixes|suffixesadd|sw|swapfile|swapsync|swb|swf|switchbuf|sws|sxq|syn|synmaxcol|syntax|tabline|tabpagemax|tabstop|tagbsearch|taglength|tagrelative|tagstack|tal|tb|tbi|tbidi|tbis|tbs|tenc|term|termbidi|termencoding|terse|textauto|textmode|textwidth|tgst|thesaurus|tildeop|timeout|timeoutlen|title|titlelen|titleold|titlestring|toolbar|toolbariconsize|top|tpm|tsl|tsr|ttimeout|ttimeoutlen|ttm|tty|ttybuiltin|ttyfast|ttym|ttymouse|ttyscroll|ttytype|tw|tx|uc|ul|undolevels|updatecount|updatetime|ut|vb|vbs|vdir|verbosefile|vfile|viewdir|viewoptions|viminfo|virtualedit|visualbell|vop|wak|warn|wb|wc|wcm|wd|weirdinvert|wfh|wfw|whichwrap|wi|wig|wildchar|wildcharm|wildignore|wildmenu|wildmode|wildoptions|wim|winaltkeys|window|winfixheight|winfixwidth|winheight|winminheight|winminwidth|winwidth|wiv|wiw|wm|wmh|wmnu|wmw|wop|wrap|wrapmargin|wrapscan|writeany|writebackup|writedelay|ww|noacd|noai|noakm|noallowrevins|noaltkeymap|noanti|noantialias|noar|noarab|noarabic|noarabicshape|noari|noarshape|noautochdir|noautoindent|noautoread|noautowrite|noautowriteall|noaw|noawa|nobackup|noballooneval|nobeval|nobin|nobinary|nobiosk|nobioskey|nobk|nobl|nobomb|nobuflisted|nocf|noci|nocin|nocindent|nocompatible|noconfirm|noconsk|noconskey|nocopyindent|nocp|nocscopetag|nocscopeverbose|nocst|nocsverb|nocuc|nocul|nocursorcolumn|nocursorline|nodeco|nodelcombine|nodg|nodiff|nodigraph|nodisable|noea|noeb|noed|noedcompatible|noek|noendofline|noeol|noequalalways|noerrorbells|noesckeys|noet|noex|noexpandtab|noexrc|nofen|nofk|nofkmap|nofoldenable|nogd|nogdefault|noguipty|nohid|nohidden|nohk|nohkmap|nohkmapp|nohkp|nohls|noic|noicon|noignorecase|noim|noimc|noimcmdline|noimd|noincsearch|noinf|noinfercase|noinsertmode|nois|nojoinspaces|nojs|nolazyredraw|nolbr|nolinebreak|nolisp|nolist|noloadplugins|nolpl|nolz|noma|nomacatsui|nomagic|nomh|noml|nomod|nomodeline|nomodifiable|nomodified|nomore|nomousef|nomousefocus|nomousehide|nonu|nonumber|noodev|noopendevice|nopaste|nopi|nopreserveindent|nopreviewwindow|noprompt|nopvw|noreadonly|noremap|norestorescreen|norevins|nori|norightleft|norightleftcmd|norl|norlc|noro|nors|noru|noruler|nosb|nosc|noscb|noscrollbind|noscs|nosecure|nosft|noshellslash|noshelltemp|noshiftround|noshortname|noshowcmd|noshowfulltag|noshowmatch|noshowmode|nosi|nosm|nosmartcase|nosmartindent|nosmarttab|nosmd|nosn|nosol|nospell|nosplitbelow|nosplitright|nospr|nosr|nossl|nosta|nostartofline|nostmp|noswapfile|noswf|nota|notagbsearch|notagrelative|notagstack|notbi|notbidi|notbs|notermbidi|noterse|notextauto|notextmode|notf|notgst|notildeop|notimeout|notitle|noto|notop|notr|nottimeout|nottybuiltin|nottyfast|notx|novb|novisualbell|nowa|nowarn|nowb|noweirdinvert|nowfh|nowfw|nowildmenu|nowinfixheight|nowinfixwidth|nowiv|nowmnu|nowrap|nowrapscan|nowrite|nowriteany|nowritebackup|nows|invacd|invai|invakm|invallowrevins|invaltkeymap|invanti|invantialias|invar|invarab|invarabic|invarabicshape|invari|invarshape|invautochdir|invautoindent|invautoread|invautowrite|invautowriteall|invaw|invawa|invbackup|invballooneval|invbeval|invbin|invbinary|invbiosk|invbioskey|invbk|invbl|invbomb|invbuflisted|invcf|invci|invcin|invcindent|invcompatible|invconfirm|invconsk|invconskey|invcopyindent|invcp|invcscopetag|invcscopeverbose|invcst|invcsverb|invcuc|invcul|invcursorcolumn|invcursorline|invdeco|invdelcombine|invdg|invdiff|invdigraph|invdisable|invea|inveb|inved|invedcompatible|invek|invendofline|inveol|invequalalways|inverrorbells|invesckeys|invet|invex|invexpandtab|invexrc|invfen|invfk|invfkmap|invfoldenable|invgd|invgdefault|invguipty|invhid|invhidden|invhk|invhkmap|invhkmapp|invhkp|invhls|invhlsearch|invic|invicon|invignorecase|invim|invimc|invimcmdline|invimd|invincsearch|invinf|invinfercase|invinsertmode|invis|invjoinspaces|invjs|invlazyredraw|invlbr|invlinebreak|invlisp|invlist|invloadplugins|invlpl|invlz|invma|invmacatsui|invmagic|invmh|invml|invmod|invmodeline|invmodifiable|invmodified|invmore|invmousef|invmousefocus|invmousehide|invnu|invnumber|invodev|invopendevice|invpaste|invpi|invpreserveindent|invpreviewwindow|invprompt|invpvw|invreadonly|invremap|invrestorescreen|invrevins|invri|invrightleft|invrightleftcmd|invrl|invrlc|invro|invrs|invru|invruler|invsb|invsc|invscb|invscrollbind|invscs|invsecure|invsft|invshellslash|invshelltemp|invshiftround|invshortname|invshowcmd|invshowfulltag|invshowmatch|invshowmode|invsi|invsm|invsmartcase|invsmartindent|invsmarttab|invsmd|invsn|invsol|invspell|invsplitbelow|invsplitright|invspr|invsr|invssl|invsta|invstartofline|invstmp|invswapfile|invswf|invta|invtagbsearch|invtagrelative|invtagstack|invtbi|invtbidi|invtbs|invtermbidi|invterse|invtextauto|invtextmode|invtf|invtgst|invtildeop|invtimeout|invtitle|invto|invtop|invtr|invttimeout|invttybuiltin|invttyfast|invtx|invvb|invvisualbell|invwa|invwarn|invwb|invweirdinvert|invwfh|invwfw|invwildmenu|invwinfixheight|invwinfixwidth|invwiv|invwmnu|invwrap|invwrapscan|invwrite|invwriteany|invwritebackup|invws|t_AB|t_AF|t_al|t_AL|t_bc|t_cd|t_ce|t_Ce|t_cl|t_cm|t_Co|t_cs|t_Cs|t_CS|t_CV|t_da|t_db|t_dl|t_DL|t_EI|t_F1|t_F2|t_F3|t_F4|t_F5|t_F6|t_F7|t_F8|t_F9|t_fs|t_IE|t_IS|t_k1|t_K1|t_k2|t_k3|t_K3|t_k4|t_K4|t_k5|t_K5|t_k6|t_K6|t_k7|t_K7|t_k8|t_K8|t_k9|t_K9|t_KA|t_kb|t_kB|t_KB|t_KC|t_kd|t_kD|t_KD|t_ke|t_KE|t_KF|t_KG|t_kh|t_KH|t_kI|t_KI|t_KJ|t_KK|t_kl|t_KL|t_kN|t_kP|t_kr|t_ks|t_ku|t_le|t_mb|t_md|t_me|t_mr|t_ms|t_nd|t_op|t_RI|t_RV|t_Sb|t_se|t_Sf|t_SI|t_so|t_sr|t_te|t_ti|t_ts|t_ue|t_us|t_ut|t_vb|t_ve|t_vi|t_vs|t_WP|t_WS|t_xs|t_ZH|t_ZR)\b/,
		'number': /\b(?:0x[\da-f]+|\d+(?:\.\d+)?)\b/i,
		'operator': /\|\||&&|[-+.]=?|[=!](?:[=~][#?]?)?|[<>]=?[#?]?|[*\/%?]|\b(?:is(?:not)?)\b/,
		'punctuation': /[{}[\](),;:]/
	};

	Prism.languages.wasm = {
		'comment': [
			/\(;[\s\S]*?;\)/,
			{
				pattern: /;;.*/,
				greedy: true
			}
		],
		'string': {
			pattern: /"(?:\\[\s\S]|[^"\\])*"/,
			greedy: true
		},
		'keyword': [
			{
				pattern: /\b(?:align|offset)=/,
				inside: {
					'operator': /=/
				}
			},
			{
				pattern: /\b(?:(?:f32|f64|i32|i64)(?:\.(?:abs|add|and|ceil|clz|const|convert_[su]\/i(?:32|64)|copysign|ctz|demote\/f64|div(?:_[su])?|eqz?|extend_[su]\/i32|floor|ge(?:_[su])?|gt(?:_[su])?|le(?:_[su])?|load(?:(?:8|16|32)_[su])?|lt(?:_[su])?|max|min|mul|nearest|neg?|or|popcnt|promote\/f32|reinterpret\/[fi](?:32|64)|rem_[su]|rot[lr]|shl|shr_[su]|store(?:8|16|32)?|sqrt|sub|trunc(?:_[su]\/f(?:32|64))?|wrap\/i64|xor))?|memory\.(?:grow|size))\b/,
				inside: {
					'punctuation': /\./
				}
			},
			/\b(?:anyfunc|block|br(?:_if|_table)?|call(?:_indirect)?|data|drop|elem|else|end|export|func|get_(?:global|local)|global|if|import|local|loop|memory|module|mut|nop|offset|param|result|return|select|set_(?:global|local)|start|table|tee_local|then|type|unreachable)\b/
		],
		'variable': /\$[\w!#$%&'*+\-./:<=>?@\\^_`|~]+/i,
		'number': /[+-]?\b(?:\d(?:_?\d)*(?:\.\d(?:_?\d)*)?(?:[eE][+-]?\d(?:_?\d)*)?|0x[\da-fA-F](?:_?[\da-fA-F])*(?:\.[\da-fA-F](?:_?[\da-fA-D])*)?(?:[pP][+-]?\d(?:_?\d)*)?)\b|\binf\b|\bnan(?::0x[\da-fA-F](?:_?[\da-fA-D])*)?\b/,
		'punctuation': /[()]/
	};
	(function (Prism) {

		// https://yaml.org/spec/1.2/spec.html#c-ns-anchor-property
		// https://yaml.org/spec/1.2/spec.html#c-ns-alias-node
		var anchorOrAlias = /[*&][^\s[\]{},]+/;
		// https://yaml.org/spec/1.2/spec.html#c-ns-tag-property
		var tag = /!(?:<[\w\-%#;/?:@&=+$,.!~*'()[\]]+>|(?:[a-zA-Z\d-]*!)?[\w\-%#;/?:@&=+$.~*'()]+)?/;
		// https://yaml.org/spec/1.2/spec.html#c-ns-properties(n,c)
		var properties = '(?:' + tag.source + '(?:[ \t]+' + anchorOrAlias.source + ')?|'
			+ anchorOrAlias.source + '(?:[ \t]+' + tag.source + ')?)';
		// https://yaml.org/spec/1.2/spec.html#ns-plain(n,c)
		// This is a simplified version that doesn't support "#" and multiline keys
		// All these long scarry character classes are simplified versions of YAML's characters
		var plainKey = /(?:[^\s\x00-\x08\x0e-\x1f!"#%&'*,\-:>?@[\]`{|}\x7f-\x84\x86-\x9f\ud800-\udfff\ufffe\uffff]|[?:-]<PLAIN>)(?:[ \t]*(?:(?![#:])<PLAIN>|:<PLAIN>))*/.source
			.replace(/<PLAIN>/g, function () { return /[^\s\x00-\x08\x0e-\x1f,[\]{}\x7f-\x84\x86-\x9f\ud800-\udfff\ufffe\uffff]/.source; });
		var string = /"(?:[^"\\\r\n]|\\.)*"|'(?:[^'\\\r\n]|\\.)*'/.source;

		/**
		 *
		 * @param {string} value
		 * @param {string} [flags]
		 * @returns {RegExp}
		 */
		function createValuePattern(value, flags) {
			flags = (flags || '').replace(/m/g, '') + 'm'; // add m flag
			var pattern = /([:\-,[{]\s*(?:\s<<prop>>[ \t]+)?)(?:<<value>>)(?=[ \t]*(?:$|,|]|}|(?:[\r\n]\s*)?#))/.source
				.replace(/<<prop>>/g, function () { return properties; }).replace(/<<value>>/g, function () { return value; });
			return RegExp(pattern, flags);
		}

		Prism.languages.yaml = {
			'scalar': {
				pattern: RegExp(/([\-:]\s*(?:\s<<prop>>[ \t]+)?[|>])[ \t]*(?:((?:\r?\n|\r)[ \t]+)\S[^\r\n]*(?:\2[^\r\n]+)*)/.source
					.replace(/<<prop>>/g, function () { return properties; })),
				lookbehind: true,
				alias: 'string'
			},
			'comment': /#.*/,
			'key': {
				pattern: RegExp(/((?:^|[:\-,[{\r\n?])[ \t]*(?:<<prop>>[ \t]+)?)<<key>>(?=\s*:\s)/.source
					.replace(/<<prop>>/g, function () { return properties; })
					.replace(/<<key>>/g, function () { return '(?:' + plainKey + '|' + string + ')'; })),
				lookbehind: true,
				greedy: true,
				alias: 'atrule'
			},
			'directive': {
				pattern: /(^[ \t]*)%.+/m,
				lookbehind: true,
				alias: 'important'
			},
			'datetime': {
				pattern: createValuePattern(/\d{4}-\d\d?-\d\d?(?:[tT]|[ \t]+)\d\d?:\d{2}:\d{2}(?:\.\d*)?(?:[ \t]*(?:Z|[-+]\d\d?(?::\d{2})?))?|\d{4}-\d{2}-\d{2}|\d\d?:\d{2}(?::\d{2}(?:\.\d*)?)?/.source),
				lookbehind: true,
				alias: 'number'
			},
			'boolean': {
				pattern: createValuePattern(/true|false/.source, 'i'),
				lookbehind: true,
				alias: 'important'
			},
			'null': {
				pattern: createValuePattern(/null|~/.source, 'i'),
				lookbehind: true,
				alias: 'important'
			},
			'string': {
				pattern: createValuePattern(string),
				lookbehind: true,
				greedy: true
			},
			'number': {
				pattern: createValuePattern(/[+-]?(?:0x[\da-f]+|0o[0-7]+|(?:\d+(?:\.\d*)?|\.?\d+)(?:e[+-]?\d+)?|\.inf|\.nan)/.source, 'i'),
				lookbehind: true
			},
			'tag': tag,
			'important': anchorOrAlias,
			'punctuation': /---|[:[\]{}\-,|>?]|\.\.\./
		};

		Prism.languages.yml = Prism.languages.yaml;

	}(Prism));

	(function (Prism) {

		function literal(str) {
			return function () { return str; };
		}

		var keyword = /\b(?:align|allowzero|and|asm|async|await|break|cancel|catch|comptime|const|continue|defer|else|enum|errdefer|error|export|extern|fn|for|if|inline|linksection|nakedcc|noalias|null|or|orelse|packed|promise|pub|resume|return|stdcallcc|struct|suspend|switch|test|threadlocal|try|undefined|union|unreachable|usingnamespace|var|volatile|while)\b/;

		var IDENTIFIER = '\\b(?!' + keyword.source + ')(?!\\d)\\w+\\b';
		var ALIGN = /align\s*\((?:[^()]|\([^()]*\))*\)/.source;
		var PREFIX_TYPE_OP = /(?:\?|\bpromise->|(?:\[[^[\]]*\]|\*(?!\*)|\*\*)(?:\s*<ALIGN>|\s*const\b|\s*volatile\b|\s*allowzero\b)*)/.source.replace(/<ALIGN>/g, literal(ALIGN));
		var SUFFIX_EXPR = /(?:\bpromise\b|(?:\berror\.)?<ID>(?:\.<ID>)*(?!\s+<ID>))/.source.replace(/<ID>/g, literal(IDENTIFIER));
		var TYPE = '(?!\\s)(?:!?\\s*(?:' + PREFIX_TYPE_OP + '\\s*)*' + SUFFIX_EXPR + ')+';

		/*
		 * A simplified grammar for Zig compile time type literals:
		 *
		 * TypeExpr = ( "!"? PREFIX_TYPE_OP* SUFFIX_EXPR )+
		 *
		 * SUFFIX_EXPR = ( \b "promise" \b | ( \b "error" "." )? IDENTIFIER ( "." IDENTIFIER )* (?! \s+ IDENTIFIER ) )
		 *
		 * PREFIX_TYPE_OP = "?"
		 *                | \b "promise" "->"
		 *                | ( "[" [^\[\]]* "]" | "*" | "**" ) ( ALIGN | "const" \b | "volatile" \b | "allowzero" \b )*
		 *
		 * ALIGN = "align" "(" ( [^()] | "(" [^()]* ")" )* ")"
		 *
		 * IDENTIFIER = \b (?! KEYWORD ) [a-zA-Z_] \w* \b
		 *
		*/

		Prism.languages.zig = {
			'comment': [
				{
					pattern: /\/{3}.*/,
					alias: 'doc-comment'
				},
				/\/{2}.*/
			],
			'string': [
				{
					// "string" and c"string"
					pattern: /(^|[^\\@])c?"(?:[^"\\\r\n]|\\.)*"/,
					lookbehind: true,
					greedy: true
				},
				{
					// multiline strings and c-strings
					pattern: /([\r\n])([ \t]+c?\\{2}).*(?:(?:\r\n?|\n)\2.*)*/,
					lookbehind: true,
					greedy: true
				},
				{
					// characters 'a', '\n', '\xFF', '\u{10FFFF}'
					pattern: /(^|[^\\])'(?:[^'\\\r\n]|\\(?:.|x[a-fA-F\d]{2}|u\{[a-fA-F\d]{1,6}\}))'/,
					lookbehind: true,
					greedy: true
				}
			],
			'builtin': /\B@(?!\d)\w+(?=\s*\()/,
			'label': {
				pattern: /(\b(?:break|continue)\s*:\s*)\w+\b|\b(?!\d)\w+\b(?=\s*:\s*(?:\{|while\b))/,
				lookbehind: true
			},
			'class-name': [
				// const Foo = struct {};
				/\b(?!\d)\w+(?=\s*=\s*(?:(?:extern|packed)\s+)?(?:enum|struct|union)\s*[({])/,
				{
					// const x: i32 = 9;
					// var x: Bar;
					// fn foo(x: bool, y: f32) void {}
					pattern: RegExp(/(:\s*)<TYPE>(?=\s*(?:<ALIGN>\s*)?[=;,)])|<TYPE>(?=\s*(?:<ALIGN>\s*)?\{)/.source.replace(/<TYPE>/g, literal(TYPE)).replace(/<ALIGN>/g, literal(ALIGN))),
					lookbehind: true,
					inside: null // see below
				},
				{
					// extern fn foo(x: f64) f64; (optional alignment)
					pattern: RegExp(/(\)\s*)<TYPE>(?=\s*(?:<ALIGN>\s*)?;)/.source.replace(/<TYPE>/g, literal(TYPE)).replace(/<ALIGN>/g, literal(ALIGN))),
					lookbehind: true,
					inside: null // see below
				}
			],
			'builtin-types': {
				pattern: /\b(?:anyerror|bool|c_u?(?:short|int|long|longlong)|c_longdouble|c_void|comptime_(?:float|int)|[iu](?:8|16|32|64|128|size)|f(?:16|32|64|128)|noreturn|type|void)\b/,
				alias: 'keyword'
			},
			'keyword': keyword,
			'function': /\b(?!\d)\w+(?=\s*\()/,
			'number': /\b(?:0b[01]+|0o[0-7]+|0x[a-fA-F\d]+(?:\.[a-fA-F\d]*)?(?:[pP][+-]?[a-fA-F\d]+)?|\d+(?:\.\d*)?(?:[eE][+-]?\d+)?)\b/,
			'boolean': /\b(?:false|true)\b/,
			'operator': /\.[*?]|\.{2,3}|[-=]>|\*\*|\+\+|\|\||(?:<<|>>|[-+*]%|[-+*/%^&|<>!=])=?|[?~]/,
			'punctuation': /[.:,;(){}[\]]/
		};

		Prism.languages.zig['class-name'].forEach(function (obj) {
			if (obj.inside === null) {
				obj.inside = Prism.languages.zig;
			}
		});

	}(Prism));
}
