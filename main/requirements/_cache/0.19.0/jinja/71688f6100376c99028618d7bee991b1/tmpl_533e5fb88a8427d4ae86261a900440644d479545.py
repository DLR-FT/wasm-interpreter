from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = '_res/svg_ico16_wait.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    pass
    yield '<svg class="svg_icon" width="16px" height="16px" viewBox="0 0 16 16">\n  <style>\n    @keyframes sdoc_wait_spin {\n      from { transform: rotate(0deg); }\n      to { transform: rotate(360deg); }\n    }\n    .sdoc_wait_hand {\n      transform-box: view-box;\n      transform-origin: 50% 50%;\n      animation: sdoc_wait_spin 0.5s linear infinite;\n    }\n  </style>\n  <circle cx="8" cy="8" r="5"></circle>\n  <circle cx="8" cy="8" r="0.5" fill="currentColor" stroke="none"></circle>\n  <g class="sdoc_wait_hand">\n    <line x1="8" y1="8" x2="8" y2="4"></line>\n  </g>\n</svg>'

blocks = {}
debug_info = ''