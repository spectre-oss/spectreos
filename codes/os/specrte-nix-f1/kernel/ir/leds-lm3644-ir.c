// SPDX-License-Identifier: GPL-2.0-only
/* Beryllium diagnostic: stock IR register settings, bounded explicit pulses. */
#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/i2c.h>
#include <linux/module.h>
#include <linux/mutex.h>
#include <linux/pm.h>
#include <linux/pwm.h>
#include <linux/regmap.h>

struct poco_ir {
 struct regmap *map;
 struct gpio_desc *enable;
 struct pwm_device *pwm;
 struct mutex lock;
 bool suspended;
};
static void poco_ir_off(struct poco_ir *ir)
{
 struct pwm_state state;
 regmap_write(ir->map, 1, 0);
 pwm_get_state(ir->pwm, &state);
 state.enabled = false;
 pwm_apply_might_sleep(ir->pwm, &state);
 gpiod_set_value_cansleep(ir->enable, 0);
}
static ssize_t pulse_ms_store(struct device *dev, struct device_attribute *attr,
                             const char *buf, size_t count)
{
 struct poco_ir *ir = dev_get_drvdata(dev);
 struct pwm_state state;
 unsigned int ms, flag1, flag2;
 int ret;
 if (kstrtouint(buf, 10, &ms) || !ms || ms > 500)
  return -EINVAL;
 mutex_lock(&ir->lock);
 if (ir->suspended) { ret = -EBUSY; goto unlock; }
 gpiod_set_value_cansleep(ir->enable, 1);
 msleep(10);
 ret = regmap_read(ir->map, 10, &flag1);
 if (ret) goto off;
 ret = regmap_read(ir->map, 11, &flag2);
 if (ret) goto off;
 if (flag1 || flag2) { ret = -EIO; goto off; }
 ret = regmap_update_bits(ir->map, 7, BIT(2), BIT(2));
 if (ret) goto off;
 ret = regmap_write(ir->map, 3, 72 | BIT(7));
 if (ret) goto off;
 ret = regmap_write(ir->map, 4, 72);
 if (ret) goto off;
 pwm_init_state(ir->pwm, &state);
 state.period = 25000000;
 state.duty_cycle = 2500000;
 state.enabled = true;
 ret = pwm_apply_might_sleep(ir->pwm, &state);
 if (ret) goto off;
 /* IR mode, both outputs, level-triggered external strobe, as in stock. */
 ret = regmap_write(ir->map, 1, 0x27);
 if (ret) goto off;
 msleep(ms);
 ret = 0;
off:
 poco_ir_off(ir);
unlock:
 mutex_unlock(&ir->lock);
 return ret ? ret : count;
}
static DEVICE_ATTR_WO(pulse_ms);
static struct attribute *poco_ir_attrs[] = { &dev_attr_pulse_ms.attr, NULL };
ATTRIBUTE_GROUPS(poco_ir);
static void poco_ir_cleanup(void *data)
{
 struct poco_ir *ir = data;
 mutex_lock(&ir->lock);
 ir->suspended = true;
 poco_ir_off(ir);
 mutex_unlock(&ir->lock);
}
static int poco_ir_probe(struct i2c_client *client)
{
 static const struct regmap_config config = { .reg_bits = 8, .val_bits = 8, .max_register = 13 };
 struct device *dev = &client->dev;
 struct poco_ir *ir;
 struct gpio_desc *gpio;
 unsigned int id;
 int ret;
 ir = devm_kzalloc(dev, sizeof(*ir), GFP_KERNEL);
 if (!ir) return -ENOMEM;
 mutex_init(&ir->lock);
 dev_set_drvdata(dev, ir);
 ir->enable = devm_gpiod_get(dev, "enable", GPIOD_OUT_LOW);
 if (IS_ERR(ir->enable)) return dev_err_probe(dev, PTR_ERR(ir->enable), "enable GPIO\n");
 gpio = devm_gpiod_get(dev, "torch", GPIOD_OUT_LOW);
 if (IS_ERR(gpio)) return dev_err_probe(dev, PTR_ERR(gpio), "torch GPIO\n");
 gpio = devm_gpiod_get(dev, "tx", GPIOD_OUT_LOW);
 if (IS_ERR(gpio)) return dev_err_probe(dev, PTR_ERR(gpio), "TX GPIO\n");
 ir->pwm = devm_pwm_get(dev, NULL);
 if (IS_ERR(ir->pwm)) return dev_err_probe(dev, PTR_ERR(ir->pwm), "PWM\n");
 ir->map = devm_regmap_init_i2c(client, &config);
 if (IS_ERR(ir->map)) return PTR_ERR(ir->map);
 ret = devm_add_action_or_reset(dev, poco_ir_cleanup, ir);
 if (ret) return ret;
 gpiod_set_value_cansleep(ir->enable, 1);
 msleep(10);
 ret = regmap_read(ir->map, 12, &id);
 poco_ir_off(ir);
 if (ret) return dev_err_probe(dev, ret, "chip ID\n");
 if (id != 2 && id != 4) return dev_err_probe(dev, -ENODEV, "unknown chip ID %u\n", id);
 dev_info(dev, "LM3644 IR ID=%u; illumination off; maximum test pulse 500 ms\n", id);
 return 0;
}
static int poco_ir_suspend(struct device *dev)
{
 poco_ir_cleanup(dev_get_drvdata(dev));
 return 0;
}
static int poco_ir_resume(struct device *dev)
{
 struct poco_ir *ir = dev_get_drvdata(dev);
 mutex_lock(&ir->lock);ir->suspended = false;mutex_unlock(&ir->lock);
 return 0;
}
static DEFINE_SIMPLE_DEV_PM_OPS(poco_ir_pm, poco_ir_suspend, poco_ir_resume);
static void poco_ir_shutdown(struct i2c_client *client) { poco_ir_cleanup(i2c_get_clientdata(client)); }
static const struct of_device_id poco_ir_match[] = { { .compatible = "spectre,beryllium-ir-diagnostic" }, {} };
MODULE_DEVICE_TABLE(of, poco_ir_match);
static struct i2c_driver poco_ir_driver = {
 .driver = { .name = "poco-ir", .dev_groups = poco_ir_groups, .of_match_table = poco_ir_match, .pm = pm_sleep_ptr(&poco_ir_pm) },
 .probe = poco_ir_probe, .shutdown = poco_ir_shutdown,
};
module_i2c_driver(poco_ir_driver);
MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Bounded Poco F1 LM3644 IR diagnostic");
