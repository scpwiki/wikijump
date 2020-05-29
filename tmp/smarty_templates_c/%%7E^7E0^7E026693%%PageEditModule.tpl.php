<?php /* Smarty version 2.6.7, created on 2008-12-06 17:19:17
         compiled from /var/www/wikidot/templates/modules/edit/PageEditModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/edit/PageEditModule.tpl', 2, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/edit/PageEditModule.tpl', 13, false),array('function', 'printuser', '/var/www/wikidot/templates/modules/edit/PageEditModule.tpl', 86, false),)), $this); ?>
<?php if ($this->_tpl_vars['newPage']): ?>
	<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Create a new page<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h1>
<?php else: ?>
	<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Edit the page<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h1>
<?php endif; ?>

<?php if ($this->_tpl_vars['lock']): ?>

<?php else: ?>
	<div>
		
		<form id="edit-page-form">
			<input type="hidden" name="page_id" value="<?php echo ((is_array($_tmp=$this->_tpl_vars['pageId'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
"/>
			<?php if ($this->_tpl_vars['mode'] == 'page' || ( $this->_tpl_vars['newPage'] && $this->_tpl_vars['templates'] )): ?>
				<table class="form" style="margin: 0.5em auto 1em 0">
					<?php if ($this->_tpl_vars['mode'] == 'page'): ?>
						<tr>
							<td>
								<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Title of the page<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
							</td>
							<td>
								<input class="text" id="edit-page-title" name="title" type="text" value="<?php echo ((is_array($_tmp=$this->_tpl_vars['title'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
" size="35" maxlength="128" 
									style="font-weight: bold; font-size: 130%;"/>
							</td>
						</tr>
					<?php endif; ?>
					<?php if ($this->_tpl_vars['newPage'] && $this->_tpl_vars['templates']): ?>
						<tr>
							<td>
								<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Initial content<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
							</td>
							<td>
								<select name="theme" id="page-templates" onchange="WIKIDOT.modules.PageEditModule.listeners.templateChange(event)">
									<option value=""  style="padding: 0 1em">no template (blank page)</option>
									<?php if (count($_from = (array)$this->_tpl_vars['templates'])):
    foreach ($_from as $this->_tpl_vars['template']):
?>	
										<option value="<?php echo $this->_tpl_vars['template']->getPageId(); ?>
"  style="padding: 0 1em" <?php if ($this->_tpl_vars['template']->getPageId() == $this->_tpl_vars['templateId']): ?>selected="selected"<?php endif; ?>><?php echo ((is_array($_tmp=$this->_tpl_vars['template']->getTitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</option>
									<?php endforeach; endif; unset($_from); ?>
								</select>
							</td>
						</tr>
					<?php endif; ?>
				</table>
			<?php endif; ?>
			<div class="wd-editor-toolbar-panel" id="wd-editor-toolbar-panel"></div>
			<div>
				<textarea id="edit-page-textarea" name="source" rows="20" cols="40" style="width: 95%;"><?php echo ((is_array($_tmp=$this->_tpl_vars['source'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</textarea>
			</div>
			<div class="change-textarea-size">
				<a href="javascript:;" onclick="WIKIDOT.utils.changeTextareaRowNo('edit-page-textarea',-5)">-</a>
				<a href="javascript:;" onclick="WIKIDOT.utils.changeTextareaRowNo('edit-page-textarea',5)">+</a>
			</div>
			<div class="edit-help-34">
				<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Need help? Check the<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> <a href="<?php echo $this->_tpl_vars['URL_DOCS']; ?>
" target="_blank"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>documentation<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>.
			</div>
		
			<table style="padding: 2px 0; border: none;">
				<tr>
					<td style="border: none; padding: 0 5px;">
						<div >
							<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Short description of changes<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
							<br/>
							<textarea id="edit-page-comments" name="comments" rows="3" cols="40" ></textarea>
						</div>
						<div class="sub">
							<?php $this->_tag_stack[] = array('t', array('escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>max 200 characters (<span id="comments-charleft"></span> left)<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
						</div>
					</td>
					<td style="border: none; padding: 0 5px;">
						<div id="lock-info" <?php if ($this->_tpl_vars['disableLocks']): ?>style="display: none"<?php endif; ?>>
							<?php $this->_tag_stack[] = array('t', array('1' => '900','escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>You have acquired an exclusive 15-minute page lock which means nobody else can edit the page simultaneously to
								avoid conflicts.
								<br/>
								The lock will expire in <strong><span id="lock-timer">%1</span></strong> seconds of inactivity.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
						</div>
					</td>
				</tr>
			</table>
			
			<?php if ($this->_tpl_vars['anonymousString']): ?>
				<div class="note-block">
					<h3><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Anonymous edit!<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h3>
					<p>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>You are editing this page content as an anonymous user. 
						Please remember that in such a case your IP address will be revealed to public
						and the changes will be signed by the following identity:<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?><br/>
						<?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['anonymousString'],'image' => 'true'), $this);?>

					</p>
				</div>
			<?php endif; ?>
			
			<div class="buttons alignleft">
				<input type="button" name="cancel" id="edit-cancel-button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>cancel<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.PageEditModule.listeners.cancel(event)"/>
				<?php if (! $this->_tpl_vars['newPage'] && $this->_tpl_vars['mode'] != 'append'): ?><input type="button" name="diff" id="edit-diff-button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>view diff<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.PageEditModule.listeners.viewDiff(event)"/><?php endif; ?>
				<input type="button" name="preview" id="edit-preview-button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>preview<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.PageEditModule.listeners.preview(event)"/>
				<?php if (! $this->_tpl_vars['newPage'] && $this->_tpl_vars['mode'] == 'page'): ?><input type="button" name="save-continue" id="edit-save-continue-button"  value="<?php $this->_tag_stack[] = array('t', array('escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>save &amp; continue<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.PageEditModule.listeners.saveAndContinue(event)"/><?php endif; ?>
				<input type="button" name="save" id="edit-save-button"  value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>save<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.PageEditModule.listeners.save(event)"/>
			</div>
		</form>
	
	
	</div>
	
	<div id="view-diff-div"></div>
	
	<div id="preview-message" style="display: none">
		<div class="preview-message">
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>This is a preview only!!!<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?><br/>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>If you leave this page now, all the changes will be lost.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?><br/>
			<a href="javascript:;" onclick="OZONE.visuals.scrollTo('action-area')"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>down to edit<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a> |
			<a href="javascript:;" onclick="document.getElementById('action-area-top').innerHTML=''"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>close this box<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
		</div>
	</div>
<?php endif; ?>