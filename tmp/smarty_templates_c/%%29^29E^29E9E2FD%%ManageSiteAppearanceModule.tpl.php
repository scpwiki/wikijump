<?php /* Smarty version 2.6.7, created on 2008-12-06 17:49:54
         compiled from /var/www/wikidot/templates/modules/managesite/ManageSiteAppearanceModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/managesite/ManageSiteAppearanceModule.tpl', 1, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/managesite/ManageSiteAppearanceModule.tpl', 16, false),)), $this); ?>
<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Themes<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h1>

<p>
	Below you can choose a theme for your site. You can select individual theme for each
	of page categories.
</p>	

<div id="sm-appearance-form-div">
	<form id="sm-appearance-form">
		
		<table class="sm-theme-table">
			<tr>
				<td>
					<select name="category" size="15" id="sm-appearance-cats">
						<?php if (count($_from = (array)$this->_tpl_vars['categories'])):
    foreach ($_from as $this->_tpl_vars['category']):
?>
							<option value="<?php echo $this->_tpl_vars['category']->getCategoryId(); ?>
" style="padding: 0 1em" <?php if ($this->_tpl_vars['category']->getName() == '_default'): ?>selected="selected"<?php endif; ?>><?php echo ((is_array($_tmp=$this->_tpl_vars['category']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</option>
						<?php endforeach; endif; unset($_from); ?>
					</select>
				</td>
				<td>
					<div id="sm-appearance-theme">
					<table>
						<tr>
							<td style="padding-right: 2em;">
					
					
					
						<h3><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Choose a built-in theme<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:</h3>
						<select name="theme" id="sm-appearance-theme-id">
							<?php if (count($_from = (array)$this->_tpl_vars['themes'])):
    foreach ($_from as $this->_tpl_vars['theme']):
?>
								<option value="<?php echo $this->_tpl_vars['theme']->getThemeId(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['theme']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
 <?php if ($this->_tpl_vars['theme']->getCustom()): ?>(<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>custom<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>)<?php endif; ?></option>
							<?php endforeach; endif; unset($_from); ?>
						</select>
						<div id="theme-variants-container">
							<?php if ($this->_tpl_vars['variantsArray']): ?>
								<?php if (count($_from = (array)$this->_tpl_vars['variantsArray'])):
    foreach ($_from as $this->_tpl_vars['variantId'] => $this->_tpl_vars['variants']):
?>
									<div id="sm-appearance-variants-<?php echo $this->_tpl_vars['variantId']; ?>
" style="display: none">
										<br/><br/><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Available theme variants<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
										<br/><br/>
										<select name="variants" id="sm-appearance-variants-select-<?php echo $this->_tpl_vars['variantId']; ?>
"
											onclick="WIKIDOT.modules.ManagerSiteAppearanceModule.listeners.variantChange(event)">
											<?php if (count($_from = (array)$this->_tpl_vars['variants'])):
    foreach ($_from as $this->_tpl_vars['variant']):
?>
												<option value="<?php echo $this->_tpl_vars['variantId']; ?>
"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Default<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></option>
												<option value="<?php echo $this->_tpl_vars['variant']->getThemeId(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['variant']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</option>
											<?php endforeach; endif; unset($_from); ?>
										</select>
									</div>
								<?php endforeach; endif; unset($_from); ?>
							<?php endif; ?>
						</div>
					</div>
							</td>
							<td style="padding-left: 2em; border-left: 1px solid #BBB;">
								<h3><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Or use an external theme<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:</h3>
								<p>
									Pass the exact URL to the location of the CSS file:
								</p>
								<input type="text" class="text" size="36" id="sm-appearance-external-url" name="sm-appearance-external-url"/>
								<p>
									Visit <a href="http://themes.wikidot.com">themes.wikidot.com</a> for the central repository of
									user-submitted themes.
								</p>
							</td>
						</tr>
					</table>
					</div>
					<div id="sm-appearance-noind" style="margin-top: 1em">
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Inherit from <tt>_default</tt><?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>: <input class="checkbox" type="checkbox" id="sm-appearance-noin"/>
					</div>
				</td>
			</tr>
		</table>
		
	
		
		<div id="sm-appearance-theme-preview" style="overflow: hidden;">
			<h2><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Theme details<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:</h2>
			
			<?php if (count($_from = (array)$this->_tpl_vars['themes'])):
    foreach ($_from as $this->_tpl_vars['theme']):
?>
				<?php $this->assign('preview', $this->_tpl_vars['theme']->getThemePreview()); ?>
				<?php if ($this->_tpl_vars['preview']): ?>
					<div id="sm-theme-preview-<?php echo $this->_tpl_vars['theme']->getThemeId(); ?>
">
						<?php echo $this->_tpl_vars['preview']->getBody(); ?>

					</div>
				<?php endif; ?>
			<?php endforeach; endif; unset($_from); ?>
		</div>
		<div class="buttons">
			<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>cancel<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" id="sm-appearance-cancel"/>
			<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>save changes<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" id="sm-appearance-save"/>
		</div>
	</form>
</div>