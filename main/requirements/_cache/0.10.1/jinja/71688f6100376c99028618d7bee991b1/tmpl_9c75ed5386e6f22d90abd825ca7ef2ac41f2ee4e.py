from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/form/field/text/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_field_required = resolve('field_required')
    l_0_data_controller = resolve('data_controller')
    l_0_field_actions = resolve('field_actions')
    l_0_field_name = resolve('field_name')
    l_0_field_input_name = resolve('field_input_name')
    l_0_errors = resolve('errors')
    l_0_field_type = resolve('field_type')
    l_0_field_editable = resolve('field_editable')
    l_0_field_label = resolve('field_label')
    l_0_singleline_suffix = resolve('singleline_suffix')
    l_0_field_class_name = resolve('field_class_name')
    l_0_field_value = resolve('field_value')
    l_0_reference_mid = resolve('reference_mid')
    l_0_field_mid = resolve('field_mid')
    try:
        t_1 = environment.filters['default']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'default' found.")
    try:
        t_2 = environment.filters['length']
    except KeyError:
        @internalcode
        def t_2(*unused):
            raise TemplateRuntimeError("No filter named 'length' found.")
    try:
        t_3 = environment.tests['defined']
    except KeyError:
        @internalcode
        def t_3(*unused):
            raise TemplateRuntimeError("No test named 'defined' found.")
    try:
        t_4 = environment.tests['sameas']
    except KeyError:
        @internalcode
        def t_4(*unused):
            raise TemplateRuntimeError("No test named 'sameas' found.")
    pass
    yield '\n\n'
    if (not t_3((undefined(name='field_required') if l_0_field_required is missing else l_0_field_required))):
        pass
        l_0_field_required = False
        context.vars['field_required'] = l_0_field_required
        context.exported_vars.add('field_required')
    yield '<sdoc-form-row\n  '
    if t_3((undefined(name='data_controller') if l_0_data_controller is missing else l_0_data_controller)):
        pass
        yield 'data-controller="'
        yield escape((undefined(name='data_controller') if l_0_data_controller is missing else l_0_data_controller))
        yield '"'
    yield '>\n  <sdoc-form-row-aside>'
    if t_3((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions)):
        pass
        if (t_3(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'move_up')) and t_4(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'move_up'), True)):
            pass
            yield '<button\n          class="field_action"\n          title="Move this '
            yield escape(t_1((undefined(name='field_name') if l_0_field_name is missing else l_0_field_name), 'FIELD', True))
            yield ' up"\n          data-action-type="move_up"\n          data-js-move-up-field-action\n          data-turbo-action="replace"\n          data-turbo="false"\n          data-testid="form-move-up-'
            yield escape(t_1((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name), 'FIELD', True))
            yield '-field-action"\n        >'
            template = environment.get_template('_res/svg_ico16_move_up.jinja.html', 'components/form/field/text/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_required': l_0_field_required}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '</button>'
        if (t_3(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'move_down')) and t_4(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'move_down'), True)):
            pass
            yield '<button\n          class="field_action"\n          title="Move this '
            yield escape(t_1((undefined(name='field_name') if l_0_field_name is missing else l_0_field_name), 'FIELD', True))
            yield ' down"\n          data-action-type="move_down"\n          data-js-move-down-field-action\n          data-turbo-action="replace"\n          data-turbo="false"\n          data-testid="form-move-down-'
            yield escape(t_1((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name), 'FIELD', True))
            yield '-field-action"\n        >'
            template = environment.get_template('_res/svg_ico16_move_down.jinja.html', 'components/form/field/text/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_required': l_0_field_required}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '</button>'
    yield '</sdoc-form-row-aside>\n\n  <sdoc-form-row-main>'
    if (t_2((undefined(name='errors') if l_0_errors is missing else l_0_errors)) > 0):
        pass
        for l_1_error in (undefined(name='errors') if l_0_errors is missing else l_0_errors):
            _loop_vars = {}
            pass
            yield '<sdoc-form-error>\n        '
            yield escape(l_1_error)
            yield '\n      </sdoc-form-error>'
        l_1_error = missing
    yield '<sdoc-form-field>\n      <sdoc-contenteditable\n        data-controller="editablefield"\n        '
    if (undefined(name='field_required') if l_0_field_required is missing else l_0_field_required):
        pass
        yield 'required'
    yield '\n        role="textbox"\n        data-field-type="'
    yield escape((undefined(name='field_type') if l_0_field_type is missing else l_0_field_type))
    yield '"'
    if (t_3((undefined(name='field_editable') if l_0_field_editable is missing else l_0_field_editable)) and (not (undefined(name='field_editable') if l_0_field_editable is missing else l_0_field_editable))):
        pass
        yield 'contenteditable="false"'
    else:
        pass
        yield 'contenteditable="true"'
    yield 'id="'
    yield escape((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name))
    yield '"\n        placeholder="Enter '
    yield escape(t_1((undefined(name='field_name') if l_0_field_name is missing else l_0_field_name), 'FIELD_NAME', True))
    yield ' here..."\n        data-field-label="'
    yield escape(((undefined(name='field_label') if l_0_field_label is missing else l_0_field_label) if t_3((undefined(name='field_label') if l_0_field_label is missing else l_0_field_label)) else (undefined(name='field_name') if l_0_field_name is missing else l_0_field_name)))
    if (undefined(name='field_required') if l_0_field_required is missing else l_0_field_required):
        pass
        yield ' *'
    yield '"'
    if (((undefined(name='field_type') if l_0_field_type is missing else l_0_field_type) == 'singleline') and t_3((undefined(name='singleline_suffix') if l_0_singleline_suffix is missing else l_0_singleline_suffix))):
        pass
        yield 'data-field-suffix="'
        yield escape((undefined(name='singleline_suffix') if l_0_singleline_suffix is missing else l_0_singleline_suffix))
        yield '"'
    if t_3((undefined(name='field_class_name') if l_0_field_class_name is missing else l_0_field_class_name)):
        pass
        yield 'class="'
        yield escape((undefined(name='field_class_name') if l_0_field_class_name is missing else l_0_field_class_name))
        yield '"'
    yield 'data-testid="form-field-'
    yield escape((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name))
    yield '"\n      >'
    if True:
        pass
        yield escape((undefined(name='field_value') if l_0_field_value is missing else l_0_field_value))
    yield '</sdoc-contenteditable>'
    if ((undefined(name='field_type') if l_0_field_type is missing else l_0_field_type) == 'parent'):
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
        if True:
            pass
            yield escape((undefined(name='field_value') if l_0_field_value is missing else l_0_field_value))
        yield '</textarea>'
    yield '</sdoc-form-field>\n\n  </sdoc-form-row-main>\n\n  <sdoc-form-row-aside>'
    if t_3((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions)):
        pass
        if (t_3(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'delete')) and t_4(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'delete'), True)):
            pass
            yield '<button\n          class="field_action"\n          title="Delete this '
            yield escape(t_1((undefined(name='field_name') if l_0_field_name is missing else l_0_field_name), 'FIELD', True))
            yield '"\n          data-action-type="delete"\n          data-js-delete-field-action\n          data-turbo-action="replace"\n          data-turbo="false"\n          data-testid="form-delete-'
            yield escape(t_1((undefined(name='field_input_name') if l_0_field_input_name is missing else l_0_field_input_name), 'FIELD', True))
            yield '-field-action"\n        >'
            template = environment.get_template('_res/svg_ico16_delete.jinja.html', 'components/form/field/text/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_required': l_0_field_required}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '</button>'
        if (t_3(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'reset')) and t_4(environment.getattr((undefined(name='field_actions') if l_0_field_actions is missing else l_0_field_actions), 'reset'), True)):
            pass
            yield '<a\n          class="field_action"\n          href="/reset_uid?reference_mid='
            yield escape((undefined(name='reference_mid') if l_0_reference_mid is missing else l_0_reference_mid))
            yield '"\n          mid="'
            yield escape((undefined(name='field_mid') if l_0_field_mid is missing else l_0_field_mid))
            yield '"\n          title="Reset UID to default"\n          data-action-type="reset"\n          data-turbo-action="replace"\n          data-turbo="true"\n          data-testid="reset-uid-field-action"\n        >'
            template = environment.get_template('_res/svg_ico16_reset.jinja', 'components/form/field/text/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_required': l_0_field_required}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '</a>'
    yield '</sdoc-form-row-aside>\n</sdoc-form-row>'

blocks = {}
debug_info = '26=50&27=52&31=56&32=59&36=62&37=64&40=67&45=69&46=71&49=78&52=81&57=83&58=85&64=93&65=95&67=99&75=103&77=107&78=109&83=116&84=118&85=120&86=125&87=128&89=130&90=133&92=136&94=138&95=140&99=142&100=145&103=153&104=156&107=164&108=167&109=173&110=175&119=178&120=180&123=183&128=185&129=187&132=194&135=197&136=199&142=201'