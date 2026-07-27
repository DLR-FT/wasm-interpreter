from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/form/field/contenteditable/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_field_class_name = resolve('field_class_name')
    l_0_field_editable = resolve('field_editable')
    l_0_field_input_name = resolve('field_input_name')
    l_0_field_label = resolve('field_label')
    l_0_field_name = resolve('field_name')
    l_0_field_placeholder = resolve('field_placeholder')
    l_0_field_value = resolve('field_value')
    l_0_mid = resolve('mid')
    l_0_testid_postfix = resolve('testid_postfix')
    l_0_field_required = resolve('field_required')
    l_0_errors = resolve('errors')
    l_0_field_type = resolve('field_type')
    try:
        t_1 = environment.filters['length']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'length' found.")
    try:
        t_2 = environment.tests['defined']
    except KeyError:
        @internalcode
        def t_2(*unused):
            raise TemplateRuntimeError("No test named 'defined' found.")
    try:
        t_3 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_3(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    def macro():
        t_4 = []
        pass
        return concat(t_4)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='field_class_name') if l_0_field_class_name is missing else l_0_field_class_name)), 'field_class_name is defined', caller=caller)
    def macro():
        t_5 = []
        pass
        return concat(t_5)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='field_editable') if l_0_field_editable is missing else l_0_field_editable)), 'field_editable is defined', caller=caller)
    def macro():
        t_6 = []
        pass
        return concat(t_6)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name)), 'field_input_name is defined', caller=caller)
    def macro():
        t_7 = []
        pass
        return concat(t_7)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='field_label') if l_0_field_label is missing else l_0_field_label)), 'field_label is defined', caller=caller)
    def macro():
        t_8 = []
        pass
        return concat(t_8)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, (not t_3((undefined(name='field_label') if l_0_field_label is missing else l_0_field_label))), 'field_label is not none', caller=caller)
    def macro():
        t_9 = []
        pass
        return concat(t_9)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, (not t_2((undefined(name='field_name') if l_0_field_name is missing else l_0_field_name))), 'field_name is not defined', caller=caller)
    def macro():
        t_10 = []
        pass
        return concat(t_10)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='field_placeholder') if l_0_field_placeholder is missing else l_0_field_placeholder)), 'field_placeholder is defined', caller=caller)
    def macro():
        t_11 = []
        pass
        return concat(t_11)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, (not t_3((undefined(name='field_placeholder') if l_0_field_placeholder is missing else l_0_field_placeholder))), 'field_placeholder is not none', caller=caller)
    def macro():
        t_12 = []
        pass
        return concat(t_12)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='field_value') if l_0_field_value is missing else l_0_field_value)), 'field_value is defined', caller=caller)
    def macro():
        t_13 = []
        pass
        return concat(t_13)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='mid') if l_0_mid is missing else l_0_mid)), 'mid is defined', caller=caller)
    def macro():
        t_14 = []
        pass
        return concat(t_14)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_2((undefined(name='testid_postfix') if l_0_testid_postfix is missing else l_0_testid_postfix)), 'testid_postfix is defined', caller=caller)
    if (not t_2((undefined(name='field_required') if l_0_field_required is missing else l_0_field_required))):
        pass
        l_0_field_required = False
        context.vars['field_required'] = l_0_field_required
        context.exported_vars.add('field_required')
    yield '\n\n<sdoc-form-field>\n  <sdoc-contenteditable\n    '
    if (t_2((undefined(name='errors') if l_0_errors is missing else l_0_errors)) and t_1((undefined(name='errors') if l_0_errors is missing else l_0_errors))):
        pass
        yield 'errors="true"'
    yield '\n    '
    if (undefined(name='field_required') if l_0_field_required is missing else l_0_field_required):
        pass
        yield 'required="true"'
    yield '\n    data-controller="editablefield"\n    role="textbox"\n    data-field-type="'
    yield escape((undefined(name='field_type') if l_0_field_type is missing else l_0_field_type))
    yield '"'
    if (t_2((undefined(name='field_editable') if l_0_field_editable is missing else l_0_field_editable)) and (not (undefined(name='field_editable') if l_0_field_editable is missing else l_0_field_editable))):
        pass
        yield 'contenteditable="false"'
    else:
        pass
        yield 'contenteditable="true"'
    yield 'id="'
    yield escape((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name))
    yield '"\n    mid="'
    yield escape((undefined(name='mid') if l_0_mid is missing else l_0_mid))
    yield '"\n    placeholder="'
    yield escape((undefined(name='field_placeholder') if l_0_field_placeholder is missing else l_0_field_placeholder))
    yield '"\n    data-field-label="'
    yield escape((undefined(name='field_label') if l_0_field_label is missing else l_0_field_label))
    if (undefined(name='field_required') if l_0_field_required is missing else l_0_field_required):
        pass
        yield '&nbsp;*'
    yield '"'
    if (not t_3((undefined(name='field_class_name') if l_0_field_class_name is missing else l_0_field_class_name))):
        pass
        yield 'class="'
        yield escape((undefined(name='field_class_name') if l_0_field_class_name is missing else l_0_field_class_name))
        yield '"'
    yield 'data-testid="form-field-'
    yield escape((undefined(name='testid_postfix') if l_0_testid_postfix is missing else l_0_testid_postfix))
    yield '"\n  >'
    yield escape((undefined(name='field_value') if l_0_field_value is missing else l_0_field_value))
    yield '</sdoc-contenteditable>'
    if ((undefined(name='field_type') if l_0_field_type is missing else l_0_field_type) == 'singleline'):
        pass
        yield '<input type="hidden" name="'
        yield escape((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name))
        yield '" value="'
        yield escape((undefined(name='field_value') if l_0_field_value is missing else l_0_field_value))
        yield '" '
        if (undefined(name='field_required') if l_0_field_required is missing else l_0_field_required):
            pass
            yield 'required'
        yield '/>'
    if ((undefined(name='field_type') if l_0_field_type is missing else l_0_field_type) == 'multiline'):
        pass
        yield '<textarea hidden name="'
        yield escape((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name))
        yield '" '
        if (undefined(name='field_required') if l_0_field_required is missing else l_0_field_required):
            pass
            yield 'required'
        yield '>'
        yield escape((undefined(name='field_value') if l_0_field_value is missing else l_0_field_value))
        yield '</textarea>'
    if (t_2((undefined(name='errors') if l_0_errors is missing else l_0_errors)) and (t_1((undefined(name='errors') if l_0_errors is missing else l_0_errors)) > 0)):
        pass
        for l_1_error_ in (undefined(name='errors') if l_0_errors is missing else l_0_errors):
            _loop_vars = {}
            pass
            yield '<sdoc-form-error>\n      '
            yield escape(l_1_error_)
            yield '\n    </sdoc-form-error>'
        l_1_error_ = missing
    yield '</sdoc-form-field>'

blocks = {}
debug_info = '1=41&2=47&3=53&4=59&5=65&7=71&8=77&9=83&10=89&11=95&12=101&14=107&15=109&30=113&33=117&38=121&39=123&44=130&45=132&46=134&47=136&48=141&49=144&51=147&53=149&56=151&57=154&60=162&61=165&62=171&66=173&67=175&69=179'