from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/git/form.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    try:
        t_1 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    yield '<sdoc-form sticky diff>\n  <form\n    action="/diff"\n    method="GET"\n  >\n\n        <input\n          type="text"\n          '
    if (not t_1(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'left_revision'))):
        pass
        yield '\n          value="'
        yield escape(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'left_revision'))
        yield '"\n          '
    else:
        pass
        yield '\n          value=""\n          '
    yield '\n          placeholder="Enter the LHS revision here"\n          id="left_revision"\n          name="left_revision"\n          data-testid="diff-screen-field-lhs"\n        />\n        '
    template = environment.get_template('components/button/diff.jinja', 'screens/git/form.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n        <input\n          type="text"\n          '
    if (not t_1(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'right_revision'))):
        pass
        yield '\n          value="'
        yield escape(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'right_revision'))
        yield '"\n          '
    else:
        pass
        yield '\n          value=""\n          '
    yield '\n          placeholder="Enter the RHS revision here"\n          id="right_revision"\n          name="right_revision"\n          data-testid="diff-screen-field-rhs"\n        />\n        <input\n          type="hidden"\n          value="'
    yield escape(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'tab'))
    yield '"\n          name="tab"\n        />\n\n  </form>\n\n  '
    if (not t_1(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'error_message'))):
        pass
        yield '\n    <div class="sdoc-form-error">'
        yield escape(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'error_message'))
        yield '</div>\n  '
    yield '\n\n</sdoc-form>'

blocks = {}
debug_info = '9=19&10=22&19=28&22=35&23=38&34=44&40=46&41=49'