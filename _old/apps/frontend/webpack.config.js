const nrwlConfig = require('@nrwl/react/plugins/webpack.js');

module.exports = (config, context) => {
  nrwlConfig(config);

  return {
    ...config,
    output: {
      ...config.output,
      publicPath: './',
    },
  };
};
