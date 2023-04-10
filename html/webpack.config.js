const path = require('path');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require('copy-webpack-plugin');
const CssMinimizerPlugin = require("css-minimizer-webpack-plugin");

const projectRoot = path.join(__dirname, '.');
const buildDirectory = path.join(projectRoot, 'frontend');
const distDirectory = path.join(projectRoot, 'build');

const prod = process.env.NODE_ENV === "production";

const config = {
    performance: {
        hints: false
    },
    devServer: {
        host: '0.0.0.0',
        port: 4587
    },
    entry: {
        main: [
            path.join(buildDirectory, 'css/style.css')
        ]
    },
    output: {
        path: distDirectory,
        filename: 'dist/[name].[fullhash:6].js',
        publicPath: '/'
    },
    module: {
        rules: [
            {
                test:/\.css$/,
                use:[
                    MiniCssExtractPlugin.loader,
                    {
                        loader: 'css-loader',
                        options: {
                            url: false,
                            sourceMap: !prod,
                        }
                    },
                    {
                        loader: 'postcss-loader',
                        options: {
                            sourceMap: !prod
                        }
                    }
                ]
            }
        ]
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                {from: 'public/manifest.json'},
                {from: 'public/sitemap.xml'},
                {from: 'public/robots.txt'},
                {from: 'public/images/*.*', to: 'images/[name][ext]'}
            ]
        }),
        new HtmlWebpackPlugin({
            template: 'public/index.html',
            inject: false
        }),
        new MiniCssExtractPlugin({
            filename: 'dist/[name].[fullhash:6].css'
        })
    ],
    optimization: {
        minimize: prod,
        minimizer: [
            `...`,
            new CssMinimizerPlugin({
                minimizerOptions: {
                    preset: [
                        "default",
                        {
                            discardComments: { removeAll: true }
                        }
                    ]
                }
            })
        ]
    }
};

if(!prod) {
    config.devtool = 'cheap-module-source-map';
}

module.exports = config;
