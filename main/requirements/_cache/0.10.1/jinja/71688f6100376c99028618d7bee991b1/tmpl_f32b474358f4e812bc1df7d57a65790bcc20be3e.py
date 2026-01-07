from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/project_index/frame_form_new_document.jinja.html'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    parent_template = None
    l_0_form = missing
    pass
    parent_template = environment.get_template('components/modal/form.jinja', 'screens/project_index/frame_form_new_document.jinja.html')
    for name, parent_block in parent_template.blocks.items():
        context.blocks.setdefault(name, []).append(parent_block)
    l_0_form = 'sdoc_modal_form'
    context.vars['form'] = l_0_form
    context.exported_vars.add('form')
    yield from parent_template.root_render_func(context)

def block_modal__context(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass

def block_modal_form__header(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\nAdd new document\n'

def block_modal_form__content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    l_0_form = resolve('form')
    l_0_document_title = resolve('document_title')
    l_0_error_object = resolve('error_object')
    l_0_document_path = resolve('document_path')
    pass
    yield '\n  <form\n    id="'
    yield escape((undefined(name='form') if l_0_form is missing else l_0_form))
    yield '"  \n    action="/actions/project_index/create_document"\n    method="POST"\n    data-turbo="true"\n  >\n    <sdoc-form-grid>'
    l_1_field_type = 'singleline'
    l_1_field_class_name = ''
    l_1_field_name = 'title'
    l_1_field_label = 'document title'
    l_1_field_input_name = 'document_title'
    l_1_field_value = (undefined(name='document_title') if l_0_document_title is missing else l_0_document_title)
    l_1_errors = context.call(environment.getattr((undefined(name='error_object') if l_0_error_object is missing else l_0_error_object), 'get_errors'), 'document_title', _block_vars=_block_vars)
    pass
    template = environment.get_template('components/form/field/text/index.jinja', 'screens/project_index/frame_form_new_document.jinja.html')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'errors': l_1_errors, 'field_class_name': l_1_field_class_name, 'field_input_name': l_1_field_input_name, 'field_label': l_1_field_label, 'field_name': l_1_field_name, 'field_type': l_1_field_type, 'field_value': l_1_field_value}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    l_1_field_type = l_1_field_class_name = l_1_field_name = l_1_field_label = l_1_field_input_name = l_1_field_value = l_1_errors = missing
    l_1_field_type = 'singleline'
    l_1_singleline_suffix = '.sdoc'
    l_1_field_class_name = 'std-input-document_path'
    l_1_field_name = 'path'
    l_1_field_label = 'document path and filename'
    l_1_field_input_name = 'document_path'
    l_1_field_value = (undefined(name='document_path') if l_0_document_path is missing else l_0_document_path)
    l_1_errors = context.call(environment.getattr((undefined(name='error_object') if l_0_error_object is missing else l_0_error_object), 'get_errors'), 'document_path', _block_vars=_block_vars)
    pass
    template = environment.get_template('components/form/field/text/index.jinja', 'screens/project_index/frame_form_new_document.jinja.html')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'errors': l_1_errors, 'field_class_name': l_1_field_class_name, 'field_input_name': l_1_field_input_name, 'field_label': l_1_field_label, 'field_name': l_1_field_name, 'field_type': l_1_field_type, 'field_value': l_1_field_value, 'singleline_suffix': l_1_singleline_suffix}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    l_1_field_type = l_1_singleline_suffix = l_1_field_class_name = l_1_field_name = l_1_field_label = l_1_field_input_name = l_1_field_value = l_1_errors = missing
    yield '</sdoc-form-grid>\n  </form>\n'

blocks = {'modal__context': block_modal__context, 'modal_form__header': block_modal_form__header, 'modal_form__content': block_modal_form__content}
debug_info = '1=13&2=16&3=21&4=30&7=40&9=53&24=63&37=79'